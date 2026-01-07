#let emoji(e, size: 48pt) = text(
  font: "Noto Emoji",
  size: size,
)[#e]

#let c-card = white
#let c-muted = rgb("#374151")
#let c-line  = rgb("#cbd5e1")

#let bold(t, size: 12pt) = text(size, weight: "bold")[#t]
#let dim(t, size: 12pt) = text(size, fill: c-muted)[#t]

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

#let render-weather(data) = rect(
  fill: c-card,
  stroke: none,
  radius: 22pt,
  inset: 16pt,
)[
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

  #grid(
    columns: (auto, 30%, auto),
    gutter: 10pt,
    align: (center, center, center),
    stack(
      spacing: 10pt,
      emoji(condition-icon(data.condition), size: 72pt),
      dim(data.condition, size: 18pt),
    ),
    [],
    stack(
      spacing: 4pt,
      dim([Hum #bold(data.humidity, size: 20pt)], size: 20pt),
      bold(data.temperature, size: 64pt),
      dim([Feels #bold(data.real_feel, size: 20pt)], size: 20pt),
    ),
  )

  #v(10pt)

  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    gutter: 4pt,
    ..data.hours.map(h => hour-col(h.time, h.temperature, h.rain))
  )
]
