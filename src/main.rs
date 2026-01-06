use std::{env, io::Cursor, net::SocketAddr, sync::Arc};

use anyhow::Context;
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma};
use reqwest::Client;
use serde::Deserialize;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod render;
use crate::render::render_widget;

const DEFAULT_KINDLE_WIDTH: u32 = 1072;
const DEFAULT_KINDLE_HEIGHT: u32 = 724;

#[derive(Clone)]
struct AppState {
    client: WeatherClient,
    config: DashboardConfig,
}

#[derive(Clone)]
struct DashboardConfig {
    latitude: f64,
    longitude: f64,
    width: u32,
    height: u32,
}

impl DashboardConfig {
    fn from_env() -> Self {
        let latitude = env::var("DEFAULT_LATITUDE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(40.7128);
        let longitude = env::var("DEFAULT_LONGITUDE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(-74.0060);
        let width = env::var("DASHBOARD_WIDTH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_KINDLE_WIDTH);
        let height = env::var("DASHBOARD_HEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_KINDLE_HEIGHT);

        Self {
            latitude,
            longitude,
            width,
            height,
        }
    }

    fn coordinates(&self, params: &RenderParams) -> Coordinates {
        Coordinates {
            latitude: params.latitude.unwrap_or(self.latitude),
            longitude: params.longitude.unwrap_or(self.longitude),
        }
    }

    fn dimensions(&self, params: &RenderParams) -> (u32, u32) {
        let width = params.width.unwrap_or(self.width).max(1);
        let height = params.height.unwrap_or(self.height).max(1);
        (width, height)
    }
}

#[derive(Clone)]
struct WeatherClient {
    http: Client,
}

impl WeatherClient {
    fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    async fn fetch_weather_data(&self, coords: Coordinates) -> anyhow::Result<WeatherData> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code&hourly=temperature_2m,precipitation_probability&forecast_days=1&timeformat=unixtime&timezone=UTC",
            coords.latitude, coords.longitude
        );

        let response: OpenMeteoResponse = self
            .http
            .get(url)
            .send()
            .await
            .context("failed to call weather API")?
            .error_for_status()
            .context("weather API returned an error")?
            .json()
            .await
            .context("failed to decode weather API response")?;

        let observation_time = response
            .current
            .time
            .and_then(|ts| Utc.timestamp_opt(ts, 0).latest())
            .map(|dt| dt.with_timezone(&Local));

        let snapshot = WeatherSnapshot {
            temperature_c: response.current.temperature_2m,
            feels_like_c: response.current.apparent_temperature,
            humidity_pct: response.current.relative_humidity_2m,
            weather_code: response.current.weather_code,
            observation_time,
        };

        let forecast = self.collect_hourly_forecast(&response);

        Ok(WeatherData { snapshot, forecast })
    }

    fn collect_hourly_forecast(&self, response: &OpenMeteoResponse) -> Vec<HourlyForecast> {
        let Some(hourly) = response.hourly.as_ref() else {
            return Vec::new();
        };

        let now = Utc::now();
        let mut last_included: Option<DateTime<Utc>> = None;
        let mut periods = Vec::new();

        for ((time, temp), precipitation) in hourly
            .time
            .iter()
            .zip(hourly.temperature_2m.iter())
            .zip(hourly.precipitation_probability.iter())
        {
            let timestamp = match Utc.timestamp_opt(*time, 0).latest() {
                Some(value) => value,
                None => continue,
            };

            if timestamp < now {
                continue;
            }

            if let Some(last) = last_included {
                if timestamp.signed_duration_since(last) < Duration::hours(2) {
                    continue;
                }
            }

            periods.push(HourlyForecast {
                time: timestamp.with_timezone(&Local),
                temperature_c: *temp,
                precipitation_probability: *precipitation,
            });
            last_included = Some(timestamp);

            if periods.len() >= 4 {
                break;
            }
        }

        periods
    }
}

#[derive(Deserialize)]
struct RenderParams {
    latitude: Option<f64>,
    longitude: Option<f64>,
    #[serde(rename = "batteryLevel")]
    battery_level: Option<u8>,
    #[serde(rename = "isCharging")]
    is_charging: Option<bool>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Clone, Copy)]
struct Coordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Clone)]
struct WeatherSnapshot {
    temperature_c: f64,
    feels_like_c: f64,
    humidity_pct: f64,
    weather_code: i32,
    observation_time: Option<DateTime<Local>>,
}

#[derive(Debug, Clone)]
struct WeatherData {
    snapshot: WeatherSnapshot,
    forecast: Vec<HourlyForecast>,
}

#[derive(Debug, Clone)]
struct HourlyForecast {
    time: DateTime<Local>,
    temperature_c: f64,
    precipitation_probability: f64,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
struct IndexTemplate {
    default_latitude: f64,
    default_longitude: f64,
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct OpenMeteoResponse {
    current: OpenMeteoCurrent,
    hourly: Option<OpenMeteoHourly>,
}

#[derive(Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: f64,
    apparent_temperature: f64,
    relative_humidity_2m: f64,
    weather_code: i32,
    time: Option<i64>,
}

#[derive(Deserialize)]
struct OpenMeteoHourly {
    time: Vec<i64>,
    temperature_2m: Vec<f64>,
    precipitation_probability: Vec<f64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let state = Arc::new(AppState {
        client: WeatherClient::new(),
        config: DashboardConfig::from_env(),
    });

    let app = Router::new()
        .route("/", get(render_index))
        .route("/render", get(render_image))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 4000).into();
    info!("Starting server on {addr}");

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutting down server");
}

async fn render_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    IndexTemplate {
        default_latitude: state.config.latitude,
        default_longitude: state.config.longitude,
        width: state.config.width,
        height: state.config.height,
    }
}

async fn render_image(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RenderParams>,
) -> Result<Response, Response> {
    let coords = state.config.coordinates(&params);
    let dims = state.config.dimensions(&params);

    let weather = match state.client.fetch_weather_data(coords).await {
        Ok(data) => data,
        Err(err) => {
            error!(
                ?err,
                "failed to fetch weather; falling back to cached values"
            );
            WeatherData {
                snapshot: WeatherSnapshot {
                    temperature_c: 0.0,
                    feels_like_c: 0.0,
                    humidity_pct: 0.0,
                    weather_code: 0,
                    observation_time: None,
                },
                forecast: Vec::new(),
            }
        }
    };

    let day_label = weather
        .snapshot
        .observation_time
        .as_ref()
        .map(|ts| ts.format("%A").to_string())
        .unwrap_or_else(|| "Today".to_string());

    let typst_source = build_widget_document(
        dims,
        &weather,
        &day_label,
        params.battery_level,
        params.is_charging,
    );

    let rgba = render_widget(&typst_source, 1.0).map_err(internal_error_anyhow)?;
    let grayscale: ImageBuffer<Luma<u8>, Vec<u8>> =
        DynamicImage::ImageRgba8(rgba).into_luma8().into();

    let mut bytes: Vec<u8> = Vec::new();
    DynamicImage::ImageLuma8(grayscale)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(internal_error)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("image/png"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store, max-age=0"),
    );

    Ok((headers, bytes).into_response())
}

fn weather_description(code: &i32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 | 2 => "Mostly clear",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing rain",
        71 | 73 | 75 => "Snowfall",
        77 => "Snow grains",
        80 | 81 | 82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Unknown",
    }
}

fn build_widget_document(
    dims: (u32, u32),
    weather: &WeatherData,
    day_label: &str,
    battery_level: Option<u8>,
    is_charging: Option<bool>,
) -> String {
    let condition = weather_description(&weather.snapshot.weather_code);
    let temperature = format!("{:.0}°C", weather.snapshot.temperature_c.round());
    let feels_like = format!("{:.0}°C", weather.snapshot.feels_like_c.round());
    let humidity = format!("{:.0}%", weather.snapshot.humidity_pct.round());
    let datetime_label = weather
        .snapshot
        .observation_time
        .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "--".to_string());

    let battery = match battery_level {
        Some(level) if is_charging.unwrap_or(false) => {
            format!("Battery {level}% (charging)")
        }
        Some(level) => format!("Battery {level}%"),
        None => "Battery status unavailable".to_string(),
    };

    let updated = weather
        .snapshot
        .observation_time
        .map(|ts| format!("Updated {}", ts.format("%Y-%m-%d %H:%M")))
        .unwrap_or_else(|| "Updated --".to_string());

    let mut hourly_cards = String::new();
    for period in weather.forecast.iter().take(4) {
        hourly_cards.push_str(&format!(
            "    hour-card(\"{}\", \"{:.0}°C\", \"{:.0}%\"),\n",
            period.time.format("%I:%M %p"),
            period.temperature_c.round(),
            period.precipitation_probability.round(),
        ));
    }

    while hourly_cards.lines().count() < 4 {
        hourly_cards.push_str("    hour-card(\"--\", \"--\", \"--\"),\n");
    }

    let template = include_str!("../templates/widget.typ");

    let mut document = template.to_string();
    for (key, value) in [
        ("{width}", dims.0.to_string()),
        ("{height}", dims.1.to_string()),
        ("{day_label}", day_label.to_string()),
        ("{datetime_label}", datetime_label),
        ("{condition}", condition.to_string()),
        ("{temperature}", temperature),
        ("{feels_like}", feels_like),
        ("{humidity}", humidity),
        ("{battery}", battery),
        ("{updated}", updated),
        ("{hourly_cards}", hourly_cards),
    ] {
        document = document.replace(key, &value);
    }

    document
}

fn internal_error<E: std::error::Error>(err: E) -> Response {
    error!(?err, "internal error while processing image");
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error",
    )
        .into_response()
}

fn internal_error_anyhow(err: anyhow::Error) -> Response {
    error!(?err, "internal error while processing image");
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error",
    )
        .into_response()
}
