use std::{env, io::Cursor, net::SocketAddr, ops::Deref, sync::Arc};

use anyhow::{Context, anyhow};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, header},
    response::Response,
    routing::get,
};
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};
use chrono::{DateTime, Local, TimeZone, Utc};
use futures::future;
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, Rgba};
use reqwest::Client;
use serde::Deserialize;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const DEFAULT_KINDLE_WIDTH: u32 = 1072;
const DEFAULT_KINDLE_HEIGHT: u32 = 1448;

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
        let latitude = env
            .var("DEFAULT_LATITUDE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(40.7128);
        let longitude = env
            .var("DEFAULT_LONGITUDE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(-74.0060);
        let width = env
            .var("DASHBOARD_WIDTH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_KINDLE_WIDTH);
        let height = env
            .var("DASHBOARD_HEIGHT")
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

    async fn fetch_current_weather(&self, coords: Coordinates) -> anyhow::Result<WeatherSnapshot> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,timeformat=unixtime",
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

        Ok(WeatherSnapshot {
            temperature_c: response.current.temperature_2m,
            feels_like_c: response.current.apparent_temperature,
            humidity_pct: response.current.relative_humidity_2m,
            weather_code: response.current.weather_code,
            observation_time,
        })
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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    default_latitude: f64,
    default_longitude: f64,
    width: u32,
    height: u32,
}

#[derive(Template)]
#[template(path = "render.html")]
struct RenderTemplate {
    coords: Coordinates,
    snapshot: WeatherSnapshot,
    battery_level: Option<u8>,
    is_charging: Option<bool>,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct OpenMeteoResponse {
    current: OpenMeteoCurrent,
}

#[derive(Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: f64,
    apparent_temperature: f64,
    relative_humidity_2m: f64,
    weather_code: i32,
    time: Option<i64>,
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
        .with_state(state);

    let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
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

    let snapshot = match state.client.fetch_current_weather(coords).await {
        Ok(data) => data,
        Err(err) => {
            error!(
                ?err,
                "failed to fetch weather; falling back to cached values"
            );
            WeatherSnapshot {
                temperature_c: 0.0,
                feels_like_c: 0.0,
                humidity_pct: 0.0,
                weather_code: 0,
                observation_time: None,
            }
        }
    };

    let template = RenderTemplate {
        timestamp: snapshot
            .observation_time
            .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string()),
        coords,
        snapshot: snapshot.clone(),
        battery_level: params.battery_level,
        is_charging: params.is_charging,
    };

    let html = template.render().map_err(internal_error)?;
    let rgba = render_html_to_image(&html, dims.0, dims.1).map_err(internal_error)?;
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

fn render_html_to_image(
    html: &str,
    width: u32,
    height: u32,
) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let cfg = DocumentConfig {
        viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Light)),
        ..Default::default()
    };

    let doc = HtmlDocument::from_html(html, cfg);
    let mut renderer = VelloCpuImageRenderer::new(width, height);
    let mut buffer = Vec::new();

    renderer.render_to_vec(
        |scene| blitz_paint::paint_scene(scene, doc.inner().deref(), 1.0, width, height),
        &mut buffer,
    );

    ImageBuffer::from_vec(width, height, buffer)
        .ok_or_else(|| anyhow!("failed to build image from Blitz renderer output"))
}

fn weather_description(code: i32) -> &'static str {
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

fn internal_error<E: std::error::Error>(err: E) -> Response {
    error!(?err, "internal error while processing image");
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error",
    )
        .into_response()
}
