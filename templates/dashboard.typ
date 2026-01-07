#import "weather.typ": render-weather
#import "wanikani.typ": render-wanikani

#set page(
  width: {{ width }}pt,
  height: {{ height }}pt,
  fill: rgb("#f2f4f7"),
  margin: 12pt,
)

#set text(
  font: "DejaVu Sans",
  size: 12pt,
  fill: rgb("#111827"),
)

#let weather-data = (
  day: "{{ weather_data.day_label }}",
  datetime: "{{ weather_data.datetime_label }}",
  condition: "{{ weather_data.condition }}",
  temperature: "{{ weather_data.temperature }}",
  real_feel: "{{ weather_data.feels_like }}",
  humidity: "{{ weather_data.humidity }}",
  battery: "{{ weather_data.battery }}",
  updated: "{{ weather_data.updated }}",
  hours: (
{% for card in weather_data.hourly_cards %}
    (time: "{{ card.time }}", temperature: "{{ card.temperature }}", rain: "{{ card.rain }}"),
{% endfor %}
  ),
)

#let wanikani-data = (
  entries: (
{% for entry in wanikani_data.entries %}
    (kanji: "{{ entry.kanji }}", meaning: "{{ entry.meaning }}"),
{% endfor %}
  ),
)

#stack(
  spacing: 18pt,
  align: center,
  render-weather(weather-data),
  render-wanikani(wanikani-data),
)
