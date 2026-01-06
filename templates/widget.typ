#set page(
  width: {{width}}pt,
  height: {{height}}pt,
  fill: rgb("#eaf1fb"),
  margin: 12pt,
)

#set text(
  font: "DejaVu Sans",
  size: 11pt,
  fill: rgb("#111827"),
)

#let c-card = white
#let c-muted = rgb("#6b7280")
#let c-line = rgb("#e5e7eb")
#let c-pill = rgb("#f3f7ff")

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

#let label(t) = text(9pt, fill: c-muted)[#t]
#let bold(t, size: 11pt) = text(size, weight: "bold")[#t]

#let pill(icon, title, val) = rect(
  fill: c-pill,
  stroke: 1pt + c-line,
  radius: 12pt,
  inset: (x: 8pt, y: 10pt),
)[
  #grid(
    columns: (18pt, 1fr),
    gutter: 10pt,
    align: (left, left),
    box(width: 16pt, height: 16pt, align(center, icon)),
    stack(
      spacing: 2pt,
      label(title),
      bold(val),
    ),
  )
]

#let hour-card(time, temp, rain) = rect(
  fill: c-card,
  stroke: 1pt + c-line,
  radius: 12pt,
  inset: (x: 14pt, y: 10pt),
)[
  #stack(
    spacing: 6pt,
    bold(time, size: 10pt),
    grid(
      columns: (1fr, 1fr),
      gutter: 10pt,
      align: (left, left),
      text(10pt, fill: c-muted)[🌡️ #temp],
      text(10pt, fill: c-muted)[☔ #rain],
    ),
  )
]

#let condition-icon(condition) = if condition-emoji.keys().contains(condition) {
  condition-emoji.at(condition)
} else {
  condition-emoji.at("Unknown")
}

#let weather-card(data) = {
  rect(
    fill: c-card,
    stroke: none,
    radius: 28pt,
    inset: 28pt,
  )[
    #grid(
      columns: (1fr, 1fr),
      gutter: 12pt,
      align: (left, right),
      bold(data.day, size: 14pt),
      text(11pt, fill: c-muted)[#data.datetime],
    )

    #v(10pt)

    #grid(
      columns: (1.2fr, 1fr),
      gutter: 10pt,
      align: (left, right),
      stack(
        spacing: 6pt,
        text(12pt, fill: c-muted)[#data.condition],
        bold(data.temp, size: 40pt),
        text(11pt, fill: c-muted)[Real feel #data.real-feel],
      ),
      align(right, text(60pt)[#data.icon]),
    )

    #v(10pt)

    #grid(
      columns: (1fr, 1fr, 1fr, 1fr),
      gutter: 10pt,
      pill([#data.icon], "Conditions", data.condition),
      pill([🌡️], "Temperature", data.temp),
      pill([🌡️], "Feels Like", data.real-feel),
      pill([💧], "Humidity", data.humidity),
    )

    #v(18pt)

    #bold([#data.hourly-title])

    #v(10pt)

    #grid(
      columns: (1fr, 1fr, 1fr, 1fr),
      gutter: 10pt,
      ..data.hours.map(h => hour-card(h.time, h.temp, h.rain))
    )

    #v(10pt)

    #grid(
      columns: (1fr, 1fr),
      gutter: 12pt,
      align: (left, right),
      text(10pt, fill: c-muted)[#data.battery],
      text(10pt, fill: c-muted)[#data.updated],
    )
  ]
}

#let weather-data = (
  day: "{{ day_label }}",
  datetime: "{{ datetime_label }}",

  condition: "{{ condition }}",
  temp: "{{ temperature }}",
  real-feel: "{{ feels_like }}",
  humidity: "{{ humidity }}",
  icon: condition-icon("{{ condition }}"),

  hourly-title: "Today · Next 8 hours (every 2 hours)",

  hours: (
{% for card in hourly_cards %}
    (time: "{{ card.time }}", temp: "{{ card.temperature }}", rain: "{{ card.rain }}"),
{% endfor %}
  ),

  battery: "{{ battery }}",
  updated: "{{ updated }}",
)

#align(center, weather-card(weather-data))
