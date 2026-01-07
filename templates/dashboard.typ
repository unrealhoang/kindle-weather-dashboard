#import "weather.typ": render-weather
#import "wanikani.typ": render-wanikani

#set page(
  width: sys.inputs.width * 1pt,
  height: sys.inputs.height * 1pt,
  fill: rgb("#f2f4f7"),
  margin: 12pt,
)

#set text(
  font: "DejaVu Sans",
  size: 12pt,
  fill: rgb("#111827"),
)

#let weather-data = sys.inputs.weather-data
#let wanikani-data = sys.inputs.wanikani-data

#stack(
  spacing: 18pt,
  render-weather(weather-data),
  render-wanikani(wanikani-data),
)
