#set page(
  width: {{width}}pt,
  height: {{height}}pt,
  fill: rgb("#f2f4f7"),
  margin: 10pt,
)

#set text(
  font: "DejaVu Sans",
  size: 12pt,
  fill: rgb("#111827"),
)

#let emoji(e, size: 48pt) = text(
  font: "Noto Emoji",
  size: size,
)[#e]

#let c-card = white
#let c-muted = rgb("#374151")
#let c-line  = rgb("#cbd5e1")
#let c-pill  = rgb("#eef2f6")

#let bold(t, size: 12pt) = text(size, weight: "bold")[#t]
#let dim(t, size: 12pt) = text(size, fill: c-muted)[#t]

// Emoji mapping
#let condition-emoji = (
  "Clear sky": "☀️",
  "Mostly clear": "🌤️",
  "Overcast": "☁️",
  "Fog": "🌫️",
  "Drizzle": "🌦️",
  "Freezing drizzle": "🌧️❄️",
  "Rain": "🌧️",
  "Freezing rain": "🌧️🧊",
  "Snowfall": "🌨️",
  "Snow grains": "❄️",
  "Rain showers": "🌦️",
  "Snow showers": "🌨️",
  "Thunderstorm": "⛈️",
  "Thunderstorm with hail": "⛈️🧊",
  "Unknown": "❓",
)
#let condition-icon(c) = if condition-emoji.keys().contains(c) {
  condition-emoji.at(c)
} else {
  condition-emoji.at("Unknown")
}

// Hour card (unchanged, already good)
#let hour-col(time, temp, rain) = rect(
  fill: c-card,
  stroke: 1.2pt + c-line,
  radius: 12pt,
  inset: (x: 10pt, y: 8pt),
)[
  #stack(
    spacing: 6pt,
    [#time],
    bold(temp, size: 20pt),
    dim([☔ #bold(rain, size: 16pt)], size: 16pt),
  )
]

#let weather-card-362(data) = rect(
  fill: c-card,
  stroke: none,
  radius: 22pt,
  inset: 16pt,
)[
  // Header
  #grid(
    columns: (1fr, auto),
    gutter: 8pt,
    align: (left, right),
    stack(
      spacing: 8pt,
      bold(data.day, size: 24pt),
      dim(data.datetime, size: 12pt),
    ),
    stack(
      spacing: 2pt,
      dim(data.battery, size: 12pt),
      dim(data.updated, size: 12pt),
    ),
  )

  #v(10pt)

  // HERO BAND: emoji left, condition+feels/hum right (balanced)
  #grid(
    columns: (auto, 30%, auto),
    gutter: 10pt,
    align: (center, center, center),

    // Make emoji visually comparable to 56pt temp
    // (emoji glyphs often render smaller than Latin digits at same pt)
    stack(
      spacing: 10pt,
      emoji(data.icon, size: 72pt),
      dim(data.condition, size: 18pt),
    ),

    // spacer
    [],

    // Right block: condition same size as feels/hum
    stack(
      spacing: 4pt,
      dim([Hum #bold(data.humidity, size: 20pt)], size: 20pt),
      bold(data.temp, size: 64pt),
      dim([Feels #bold(data.real-feel, size: 20pt)], size: 20pt),
    ),
  )

  #v(10pt)

  // Hourly forecast (4 tiles)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    gutter: 4pt,
    ..data.hours.map(h => hour-col(h.time, h.temp, h.rain))
  )
]

#let weather-data = (
  day: "{{ day_label }}",
  datetime: "{{ datetime_label }}",
  condition: "{{ condition }}",
  temp: "{{ temperature }}",
  real-feel: "{{ feels_like }}",
  humidity: "{{ humidity }}",
  icon: condition-icon("{{ condition }}"),
  hours: (
{% for card in hourly_cards %}
    (time: "{{ card.time }}", temp: "{{ card.temperature }}", rain: "{{ card.rain }}"),
{% endfor %}
  ),
  battery: "{{ battery }}",
  updated: "{{ updated }}",
)

#align(center, weather-card-362(weather-data))
