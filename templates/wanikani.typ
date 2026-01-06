#set page(
  width: {{ width }}pt,
  height: {{ height }}pt,
  fill: rgb("#f2f4f7"),
  margin: 24pt,
)

#set text(
  font: "DejaVu Sans",
  size: 14pt,
  fill: rgb("#111827"),
)

#let card-bg = white
#let card-border = rgb("#cbd5e1")
#let muted = rgb("#4b5563")

#let kanji-card(entry) = rect(
  fill: card-bg,
  stroke: 1.5pt + card-border,
  radius: 14pt,
  inset: (x: 12pt, y: 10pt),
)[
  #stack(
    spacing: 6pt,
    align: center,
    text(size: 88pt, weight: "bold")[#entry.kanji],
    text(size: 12pt, fill: muted)[#entry.meaning],
  )
]

#let entries-data = (
{% for entry in entries %}
  (kanji: "{{ entry.kanji }}", meaning: "{{ entry.meaning }}"),
{% endfor %}
)

#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (1fr, 1fr),
  gutter: 10pt,
  ..entries-data.map(entry => kanji-card(entry)),
)
