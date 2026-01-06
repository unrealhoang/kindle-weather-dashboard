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

#let weather-card(
  day: "Tuesday",
  datetime: "2026-01-06 11:00",
  condition: "Clear sky",
  temp: "6°C",
  real-feel: "2°C",
  humidity: "38%",
  battery: "Battery 75% (charging)",
  updated: "Updated 2026-01-06 11:00",
) = rect(
  fill: c-card,
  stroke: none,
  radius: 28pt,
  inset: 28pt,
)[
  #grid(
    columns: (1fr, 1fr),
    gutter: 12pt,
    align: (left, right),
    bold(day, size: 14pt),
    text(11pt, fill: c-muted)[#datetime],
  )

  #v(10pt)

  #grid(
    columns: (1.2fr, 1fr),
    gutter: 10pt,
    align: (left, right),
    stack(
      spacing: 6pt,
      text(12pt, fill: c-muted)[#condition],
      bold(temp, size: 40pt),
      text(11pt, fill: c-muted)[Real feel #real-feel],
    ),
    align(right, text(60pt)[⛅]),
  )

  #v(10pt)

  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    gutter: 10pt,
    pill([⛅], "Conditions", condition),
    pill([🌡️], "Temperature", temp),
    pill([🌡️], "Feels Like", real-feel),
    pill([💧], "Humidity", humidity),
  )

  #v(18pt)

  #bold([Today · Next 8 hours (every 2 hours)])

  #v(10pt)

  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    gutter: 10pt,
{% for card in hourly_cards %}
    hour-card("{{ card.time }}", "{{ card.temperature }}", "{{ card.rain }}"),
{% endfor %}
  )

  #v(10pt)

  #grid(
    columns: (1fr, 1fr),
    gutter: 12pt,
    align: (left, right),
    text(10pt, fill: c-muted)[#battery],
    text(10pt, fill: c-muted)[#updated],
  )
]

#align(center, weather-card(
  day: "{{ day_label }}",
  datetime: "{{ datetime_label }}",
  condition: "{{ condition }}",
  temp: "{{ temperature }}",
  real-feel: "{{ feels_like }}",
  humidity: "{{ humidity }}",
  battery: "{{ battery }}",
  updated: "{{ updated }}",
))
