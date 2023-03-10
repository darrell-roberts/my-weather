use crate::{Entry, Term};
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::HashMap, marker::PhantomData, ops::Not};

use crate::parsers::{parse_current_forecast, parse_forecast};

/// Wrapper type for weather entry elements allowing
/// classifying and grouping entries.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum ForecastEntry {
  Warning(Entry),
  Current(CurrentForecastWithEntry),
  Future {
    sequence: usize,
    day: Option<ForecastWithEntry>,
    night: Option<ForecastWithEntry>,
  },
}

impl ForecastEntry {
  /// Convert html to pango doc.
  pub fn summary(&self) -> String {
    let remap_html = |input: &str| {
      input
        .replace("&deg;", "°")
        .replace("<br/>", "")
        .replace(". ", ".\n")
        .replace("minus ", "-")
        .replace("plus ", "")
    };

    match self {
      Self::Warning(entry) => remap_html(&entry.summary),
      Self::Current(CurrentForecastWithEntry { entry, .. }) => remap_html(&entry.summary),
      Self::Future { day, night, .. } => match (day, night) {
        (Some(d), Some(n)) => {
          format!(
            "<b>Day:</b>\n{}\n\n<b>Night:</b>\n{}",
            remap_html(&d.entry.summary),
            remap_html(&n.entry.summary)
          )
        }
        (Some(d), None) => {
          format!("<b>Day:</b>\n{}", remap_html(&d.entry.summary),)
        }
        (None, Some(n)) => {
          format!("<b>Night:</b>\n{}", remap_html(&n.entry.summary))
        }
        (None, None) => String::new(),
      },
    }
  }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
enum Day {
  Monday,
  Tuesday,
  Wednesday,
  Thursday,
  Friday,
  Saturday,
  Sunday,
}

fn parse_future_forecast(entry: &Entry) -> Option<(Day, Forecast)> {
  let day_key = Day::try_from(entry).ok()?;
  let forecast_entry = entry.title.as_str().parse::<Forecast>().ok()?;
  Some((day_key, forecast_entry))
}

/// Convert an iteration of weather Entry items into a Vec of ForeCastEntry, grouping
/// future forecasts by day while maintaining the original sequence.
pub fn to_forecast(entries: impl Iterator<Item = Entry>) -> Vec<ForecastEntry> {
  let mut day_map = HashMap::new();
  let mut result = vec![];

  for (index, entry) in entries.enumerate() {
    match entry.category.term {
      Term::Current => {
        result.extend(
          entry
            .title
            .as_str()
            .parse::<CurrentForecast>()
            .ok()
            .map(|current| ForecastEntry::Current(CurrentForecastWithEntry { current, entry })),
        );
      }
      Term::Warnings => result.push(ForecastEntry::Warning(entry)),
      Term::ForeCast => {
        if let Some((day_key, forecast)) = parse_future_forecast(&entry) {
          let is_day = forecast.day == DayNight::Day;
          let fc_entry = Some(ForecastWithEntry { forecast, entry });
          if let Some(ForecastEntry::Future { day, night, .. }) = day_map.get_mut(&day_key) {
            if is_day {
              *day = fc_entry;
            } else {
              *night = fc_entry;
            }
          } else {
            let (day, night) = if is_day {
              (fc_entry, None)
            } else {
              (None, fc_entry)
            };
            day_map.insert(
              day_key,
              ForecastEntry::Future {
                sequence: index,
                day,
                night,
              },
            );
          }
        }
      }
    }
  }

  // Future forecasts need to be arranged back into their original order.
  let mut sorted = day_map.into_values().collect::<Vec<_>>();
  sorted.sort_by(|a, b| match (a, b) {
    (ForecastEntry::Future { sequence: a, .. }, ForecastEntry::Future { sequence: b, .. }) => {
      a.cmp(b)
    }
    _ => std::cmp::Ordering::Equal,
  });

  result.append(&mut sorted);
  result
}

/// The day could not be parsed.
struct NotAForecastDay;

impl TryFrom<&Entry> for Day {
  type Error = NotAForecastDay;

  fn try_from(value: &Entry) -> Result<Self, Self::Error> {
    match value.category.term {
      Term::ForeCast => match &value.title {
        s if s.starts_with("Monday") => Ok(Day::Monday),
        s if s.starts_with("Tuesday") => Ok(Day::Tuesday),
        s if s.starts_with("Wednesday") => Ok(Day::Wednesday),
        s if s.starts_with("Thursday") => Ok(Day::Thursday),
        s if s.starts_with("Friday") => Ok(Day::Friday),
        s if s.starts_with("Saturday") => Ok(Day::Saturday),
        s if s.starts_with("Sunday") => Ok(Day::Sunday),
        _ => Err(NotAForecastDay),
      },
      _ => Err(NotAForecastDay),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub enum Celsius {}

#[derive(Debug, Copy, Clone)]
pub enum Fahrenheit {}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Temperature<Unit> {
  #[serde(serialize_with = "serialize_temperature")]
  High(f32, PhantomData<Unit>),
  #[serde(serialize_with = "serialize_temperature")]
  Low(f32, PhantomData<Unit>),
  #[serde(serialize_with = "serialize_temperature")]
  Current(f32, PhantomData<Unit>),
}

fn serialize_temperature<S, Unit>(
  temperature: &f32,
  _p: &PhantomData<Unit>,
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_f32(*temperature)
}

impl PartialEq for Temperature<Celsius> {
  fn eq(&self, other: &Self) -> bool {
    self == other
  }
}

impl From<Temperature<Celsius>> for Temperature<Fahrenheit> {
  fn from(value: Temperature<Celsius>) -> Self {
    let convert = |n| (n * 2.) + 30.;
    match value {
      Temperature::High(n, _) => Temperature::<Fahrenheit>::High(convert(n), PhantomData),
      Temperature::Low(n, _) => Temperature::<Fahrenheit>::Low(convert(n), PhantomData),
      Temperature::Current(n, _) => Temperature::<Fahrenheit>::Current(convert(n), PhantomData),
    }
  }
}

impl std::fmt::Display for Temperature<Celsius> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut w = |n| write!(f, "{n}°C");
    match self {
      Self::High(n, _) => w(n),
      Self::Low(n, _) => w(n),
      Self::Current(n, _) => w(n),
    }
  }
}

impl std::fmt::Display for Temperature<Fahrenheit> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut w = |n| write!(f, "{n:.0}°F");
    match self {
      Self::High(n, _) => w(n),
      Self::Low(n, _) => w(n),
      Self::Current(n, _) => w(n),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastWithEntry {
  pub forecast: Forecast,
  pub entry: Entry,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Forecast {
  pub celsius: Temperature<Celsius>,
  pub fahrenheit: Temperature<Fahrenheit>,
  pub description: String,
  pub day: DayNight,
  pub day_of_week: DayOfWeek,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayNight {
  Day,
  Night,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayOfWeek {
  Monday,
  Tuesday,
  Wednesday,
  Thursday,
  Friday,
  Saturday,
  Sunday,
}

impl DayOfWeek {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Monday => "Monday",
      Self::Tuesday => "Tuesday",
      Self::Wednesday => "Wednesday",
      Self::Thursday => "Thursday",
      Self::Friday => "Friday",
      Self::Saturday => "Saturday",
      Self::Sunday => "Sunday",
    }
  }
}

#[derive(Debug)]
pub struct TitleParseError(String);

impl std::str::FromStr for Forecast {
  type Err = TitleParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_forecast::<Celsius>(s)
      .map_err(|e| TitleParseError(e.to_string()))
      .map(|(_, forecast)| forecast)
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentForecastWithEntry {
  pub current: CurrentForecast,
  pub entry: Entry,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentForecast {
  pub celsius: Temperature<Celsius>,
  pub fahrenheit: Temperature<Fahrenheit>,
  pub description: String,
}

#[derive(Debug)]
pub struct CurrentForecastError(String);

impl std::str::FromStr for CurrentForecast {
  type Err = CurrentForecastError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_current_forecast(s)
      .map_err(|e| CurrentForecastError(e.to_string()))
      .map(|(_, cf)| cf)
  }
}
