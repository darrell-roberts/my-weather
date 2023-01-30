//! Gets weather forecast for Weather Canada RSS feed.
use serde::{
  de::{value::StringDeserializer, Visitor},
  Deserialize, Deserializer, Serialize,
};
use thiserror::Error;

mod parsers;
pub mod types;

static WEATHER_FEED: &str = "https://weather.gc.ca/rss/city/qc-58_e.xml";

/// Weather Forecast
#[derive(Debug, Serialize)]
pub struct ForeCast(Feed);

impl std::fmt::Display for ForeCast {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for entry in &self.0.entries {
      writeln!(f, "{entry}")?;
    }
    Ok(())
  }
}

impl ForeCast {
  pub fn current_forecast(&self) -> impl Iterator<Item = &Entry> {
    self
      .0
      .entries
      .iter()
      .filter(|entry| matches!(entry.category.term, Term::Current | Term::Warnings))
  }

  pub fn entries(self) -> impl Iterator<Item = Entry> {
    self.0.entries.into_iter()
  }
}

/// RSS Feed Element.
#[derive(Deserialize, Serialize, Debug)]
struct Feed {
  #[serde(rename = "entry", deserialize_with = "deserialize_entries")]
  entries: Vec<Entry>,
}

/// RSS Entry Element.
#[derive(Deserialize, Debug, Serialize)]
pub struct Entry {
  pub title: String,
  // link: String,
  // updated: String,
  pub category: Category,
  #[serde(deserialize_with = "deserialize_summary")]
  pub summary: String,
}

fn deserialize_summary<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
  struct TruncatingStringVisitor;

  impl<'de> Visitor<'de> for TruncatingStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expecting string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      v.rfind("Forecast issued")
        .map(|index| {
          let (keep, _) = v.split_at(index);
          Ok(keep.trim().into())
        })
        .unwrap_or_else(|| Ok(v.into()))
    }
  }
  deserializer.deserialize_string(TruncatingStringVisitor)
}

impl Entry {
  pub fn is_warning(&self) -> bool {
    matches!(self.category.term, Term::Warnings)
  }
}

impl std::fmt::Display for Entry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.title)
  }
}

// RSS Category Element.
#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
  pub term: Term,
}

/// RSS Category term attribute.
#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Term {
  #[serde(rename = "Current Conditions")]
  Current,
  #[serde(rename = "Weather Forecasts")]
  ForeCast,
  #[serde(rename = "Warnings and Watches")]
  Warnings,
}

/// If no watches or warnings are in effect then exclude from deserialization.
fn deserialize_entries<'de, D>(deserializer: D) -> Result<Vec<Entry>, D::Error>
where
  D: Deserializer<'de>,
{
  struct FilteredEntries;

  impl<'de> Visitor<'de> for FilteredEntries {
    type Value = Vec<Entry>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a list of Entry")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: serde::de::SeqAccess<'de>,
    {
      let mut entries = vec![];
      while let Some(next) = seq.next_element::<Entry>()? {
        // Ignore watches / warnings that are are not in effect
        if !(matches!(next.category.term, Term::Warnings)
          && next.title.starts_with("No watches or warnings in effect"))
        {
          entries.push(next);
        }
      }
      Ok(entries)
    }
  }
  deserializer.deserialize_seq(FilteredEntries)
}

/// API or Deserialization errors.
#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Rss call failed {0}")]
  Rss(#[from] reqwest::Error),
  #[error("Deserialize error {0}")]
  Parse(#[from] serde_xml_rs::Error),
}

/// Gets the weather forecast from remote RSS feed.
#[cfg(feature = "async")]
pub async fn get_weather() -> Result<ForeCast, ApiError> {
  let body = reqwest::get(WEATHER_FEED).await?.text().await?;
  Ok(ForeCast(serde_xml_rs::from_str(&body)?))
}

#[cfg(not(feature = "async"))]
pub fn get_weather() -> Result<ForeCast, ApiError> {
  let body = reqwest::blocking::get(WEATHER_FEED)?.text()?;
  Ok(ForeCast(serde_xml_rs::from_str(&body)?))
}

#[cfg(test)]
mod test {
  use super::*;

  #[cfg(feature = "async")]
  #[tokio::test]
  async fn test_api() {
    let result = get_weather().await.unwrap();
    assert!(!result.0.entries.is_empty());
  }

  #[cfg(feature = "async")]
  #[tokio::test]
  async fn test_current_forecast() {
    let result = get_weather().await.unwrap();
    let current = result.current_forecast();
    assert!(current.count() > 0);
  }

  #[cfg(not(feature = "async"))]
  #[test]
  fn test_api() {
    let result = get_weather().unwrap();
    assert!(!result.0.entries.is_empty());
  }

  #[cfg(not(feature = "async"))]
  #[test]
  fn test_current_forecast() {
    let result = get_weather().unwrap();
    let current = result.current_forecast();
    assert!(current.count() > 0);
  }
}
