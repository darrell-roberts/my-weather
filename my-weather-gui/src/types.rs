use my_weather::{Entry, Term};
use std::collections::HashMap;

/// Wrapper type for weather entry elements allowing
/// classifying and grouping entries.
#[derive(Debug)]
pub enum ForeCastEntry {
  Warning(Entry),
  Current(Entry),
  Future {
    sequence: usize,
    day: Option<Entry>,
    night: Option<Entry>,
  },
}

impl ForeCastEntry {
  pub fn summary(&self) -> String {
    let remap_html = |input: &str| {
      input
        .replace("&deg;", "°")
        .replace("<br/>", "")
        .replace(". ", ".\n")
    };

    match self {
      Self::Warning(entry) => remap_html(&entry.summary),
      Self::Current(entry) => remap_html(&entry.summary),
      Self::Future {
        day: Some(d),
        night: Some(n),
        ..
      } => {
        format!(
          "<b>Day:</b>\n{}\n\n<b>Night:</b>\n{}",
          remap_html(&d.summary),
          remap_html(&n.summary)
        )
      }
      Self::Future {
        day: Some(d),
        night: None,
        ..
      } => {
        format!("<b>Day:</b>\n{}", remap_html(&d.summary),)
      }
      Self::Future {
        day: None,
        night: Some(n),
        ..
      } => {
        format!("<b>Night:</b>\n{}", remap_html(&n.summary))
      }
      _ => String::new(),
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

/// Convert an iteration of weather Entry items into a Vec of ForeCastEntry, grouping
/// future forecasts by day while maintaining the original sequence.
pub fn to_forecast(entries: impl Iterator<Item = Entry>) -> Vec<ForeCastEntry> {
  let mut day_map = HashMap::new();
  let mut result = vec![];

  for (index, entry) in entries.enumerate() {
    match entry.category.term {
      Term::Current => result.push(ForeCastEntry::Current(entry)),
      Term::Warnings => result.push(ForeCastEntry::Warning(entry)),
      Term::ForeCast => match Day::try_from(&entry) {
        Ok(key) => {
          if let Some(ForeCastEntry::Future { night, day, .. }) =
            day_map.get_mut(&key)
          {
            let _ = std::mem::replace(
              if entry.title.contains("night:") {
                night
              } else {
                day
              },
              Some(entry),
            );
          } else {
            day_map.insert(
              key,
              if entry.title.contains("night:") {
                ForeCastEntry::Future {
                  sequence: index,
                  day: None,
                  night: Some(entry),
                }
              } else {
                ForeCastEntry::Future {
                  sequence: index,
                  day: Some(entry),
                  night: None,
                }
              },
            );
          }
        }
        Err(_) => eprintln!("No day from forecast parsed: {}", entry.title),
      },
    }
  }

  // Future forecasts need to be arranged back into their original order.
  let mut sorted = day_map.into_values().collect::<Vec<_>>();
  sorted.sort_by(|a, b| match (a, b) {
    (
      ForeCastEntry::Future { sequence: a, .. },
      ForeCastEntry::Future { sequence: b, .. },
    ) => a.cmp(b),
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
