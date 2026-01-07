#let card-bg = white
#let card-border = rgb("#cbd5e1")
#let muted = rgb("#4b5563")
#let c-card = white

#let kanji-card(entry) = rect(
  fill: card-bg,
  stroke: 1.5pt + card-border,
  radius: 14pt,
  inset: (x: 24pt, y: 24pt, bottom: 16pt),
)[
  #stack(
    spacing: 16pt,
    text(size: 110pt, font: "Noto Sans JP", weight: "bold")[#entry.kanji],
    text(size: 16pt, fill: muted)[#entry.meaning],
  )
]

#let render-wanikani(data) = rect(
  fill: c-card,
  stroke: none,
  radius: 22pt,
  inset: 16pt,
)[
  #grid(
    columns: (1fr, 1fr, 1fr),
    rows: 2,
    gutter: 10pt,
    align: center,
    ..data.entries.map(entry => kanji-card(entry)),
  )
]
