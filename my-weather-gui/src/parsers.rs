//! Parser combinator functions for parsing text into structured types.
use std::marker::PhantomData;

use crate::types::{CurrentForecast, DayNight, DayOfWeek, Forecast, Temperature};
use nom::{
  branch::alt,
  bytes::complete::{tag, take_until},
  character::complete::{char, digit1, space0},
  combinator::{map, map_res, opt, recognize, value},
  sequence::{delimited, tuple},
  IResult,
};

/// Parse an optionally signed number.
fn parse_number(input: &str) -> IResult<&str, f32> {
  let sign = recognize(tuple((
    alt((tag("minus"), tag("plus"), tag("zero"))),
    space0,
  )));

  let num_parse = delimited(
    space0,
    tuple((sign, opt(digit1))),
    alt((char('.'), char(' '))),
  );
  let mut parser = map_res(num_parse, |(sign, n): (&str, Option<&str>)| {
    if sign == "zero" {
      Ok(0.)
    } else {
      n.expect("parsed number")
        .parse::<f32>()
        .map(|num| if sign == "minus " { -num } else { num })
    }
  });
  parser(input)
}

fn parse_temp<Unit>(input: &str) -> IResult<&str, Temperature<Unit>> {
  let high_parser = map(
    delimited(
      alt((
        tag("High"),
        tag("Temperature steady near"),
        tag("Temperature rising to"),
      )),
      parse_number,
      opt(char(' ')),
    ),
    |n| Temperature::High(n, PhantomData),
  );

  let low_parser = map(delimited(tag("Low"), parse_number, opt(char(' '))), |n| {
    Temperature::Low(n, PhantomData)
  });

  alt((high_parser, low_parser))(input)
}

fn parse_description(input: &str) -> IResult<&str, &str> {
  let parse_tags = alt((
    take_until("Low"),
    take_until("High"),
    take_until("Temperature steady near"),
    take_until("Temperature rising to"),
  ));
  let mut parser = map(parse_tags, |s: &str| s.trim());
  parser(input)
}

fn parse_day_night(input: &str) -> IResult<&str, DayNight> {
  let day = value(DayNight::Day, tag(":"));
  let night = value(DayNight::Night, recognize(tuple((space0, tag("night:")))));
  let mut parser = alt((day, night));
  parser(input)
}

fn parse_day_of_week(input: &str) -> IResult<&str, DayOfWeek> {
  let mut parser = alt((
    value(DayOfWeek::Monday, tag("Monday")),
    value(DayOfWeek::Tuesday, tag("Tuesday")),
    value(DayOfWeek::Wednesday, tag("Wednesday")),
    value(DayOfWeek::Thursday, tag("Thursday")),
    value(DayOfWeek::Friday, tag("Friday")),
    value(DayOfWeek::Saturday, tag("Saturday")),
    value(DayOfWeek::Sunday, tag("Sunday")),
  ));
  parser(input)
}

/// Parses a future forecast.
pub fn parse_forecast<Unit>(input: &str) -> IResult<&str, Forecast> {
  let (input, day_of_week) = parse_day_of_week(input)?;
  let (input, day_night) = parse_day_night(input)?;
  let parser = tuple((map(parse_description, String::from), parse_temp));
  let mut parser = map(parser, |(description, temp)| Forecast {
    day: day_night,
    day_of_week,
    celsius: temp,
    fahrenheit: temp.into(),
    description,
  });
  parser(input)
}

fn parse_signed_number(input: &str) -> IResult<&str, f32> {
  let fraction_parse = recognize(tuple((digit1, char('.'), digit1)));
  let num_parse = delimited(
    space0,
    tuple((opt(char('-')), alt((fraction_parse, digit1)))),
    space0,
  );
  let mut parser = map_res(num_parse, |(neg, n): (Option<_>, &str)| {
    n.parse::<f32>()
      .map(|num| if neg.is_some() { -num } else { num })
  });
  parser(input)
}

/// Parses the current forecast.
pub fn parse_current_forecast(input: &str) -> IResult<&str, CurrentForecast> {
  let (input, description) = delimited(
    tag("Current Conditions: "),
    map(take_until(", "), String::from),
    tag(", "),
  )(input)?;
  let (input, temperature) = map(parse_signed_number, |n| {
    Temperature::Current(n, PhantomData)
  })(input)?;

  Ok((
    input,
    CurrentForecast {
      description,
      celsius: temperature,
      fahrenheit: temperature.into(),
    },
  ))
}

#[cfg(test)]
mod test {
  use crate::types::Celsius;
  use std::marker::PhantomData;

  use super::*;

  fn test_parse_entry(input: &str, expected: (String, Temperature<Celsius>)) {
    let (_, forecast) = parse_forecast::<Celsius>(input).unwrap();
    assert_eq!(forecast.description, expected.0);
    assert_eq!(forecast.celsius, expected.1); // this causes a stack overflow?
  }

  #[test]
  #[ignore]
  fn test_parse_temp() {
    test_parse_entry(
      "Saturday: A mix of sun and cloud. Temperature steady near minus 1.",
      (
        "A mix of sun and cloud.".into(),
        Temperature::<Celsius>::High(-1., PhantomData),
      ),
    );
    test_parse_entry(
      "Saturday night: A few clouds. Low minus 12.",
      (
        "A few clouds.".into(),
        Temperature::<Celsius>::Low(-12., PhantomData),
      ),
    );
    test_parse_entry(
      "Saturday night: A few clouds. Low minus 13.",
      (
        "A few clouds.".into(),
        Temperature::<Celsius>::Low(-13., PhantomData),
      ),
    )
  }

  #[test]
  fn test_parse_number() {
    let test = "minus 1. Forecast issued 3:45 PM EST Friday 06 January 2023";
    let result = parse_number(test);
    assert!(result.is_ok());

    let test = "zero.";

    let result = parse_number(test).unwrap();
    assert_eq!(result.1, 0.);
  }

  #[test]
  fn test_parse_day_of_week() {
    let test = "Monday: Sunny. High zero.";

    let result = parse_day_of_week(test).unwrap();
    assert_eq!(result.1, DayOfWeek::Monday);

    let test = "Monday night: Cloudy periods. Low minus 7.";
    let result = parse_day_of_week(test).unwrap();
    assert_eq!(result.1, DayOfWeek::Monday);
  }

  #[test]
  fn test_parse_day_night() {
    let test = " night: blah";
    let (_, day) = parse_day_night(test).unwrap();
    assert_eq!(day, DayNight::Night);

    let test = ": blah";
    let (_, day) = parse_day_night(test).unwrap();
    assert_eq!(day, DayNight::Day);
  }

  #[test]
  fn parse_full() {
    let test = "Monday: Sunny. High zero.";
    let (_, forecast) = parse_forecast::<Celsius>(test).unwrap();

    assert!(matches!(
        forecast,
        Forecast {
            celsius: Temperature::High(c, PhantomData),
            fahrenheit: Temperature::High(_, PhantomData),
            description,
            day: DayNight::Day,
            day_of_week: DayOfWeek::Monday,
        } if c == 0. && description == "Sunny."
    ));

    let test = "Sunday night: Cloudy periods. Low minus 9.";
    let (_, forecast) = parse_forecast::<Celsius>(test).unwrap();

    assert!(matches!(
        forecast,
        Forecast {
            celsius: Temperature::Low(n, PhantomData),
            fahrenheit: Temperature::Low(_, PhantomData),
            description,
            day: DayNight::Night,
            day_of_week: DayOfWeek::Sunday,
        } if n == -9. && description == "Cloudy periods."
    ));

    let test = "Thursday: Snow. High plus 2.";
    let (_, forecast) = parse_forecast::<Celsius>(test).unwrap();

    assert!(matches!(
      forecast,
      Forecast {
        celsius: Temperature::High(n, PhantomData),
        fahrenheit: Temperature::High(_, PhantomData),
        description,
        day: DayNight::Day,
        day_of_week: DayOfWeek::Thursday,
      } if n == 2. && description == "Snow."
    ));

    let test = "Saturday: Chance of flurries. High minus 3. POP 60%";
    let (_, forecast) = parse_forecast::<Celsius>(test).unwrap();

    assert!(matches!(
      forecast,
      Forecast {
        celsius: Temperature::High(n, PhantomData),
        fahrenheit: Temperature::High(_, PhantomData),
        description,
        day: DayNight::Day,
        day_of_week: DayOfWeek::Saturday,
      } if n == -3. && description == "Chance of flurries."
    ));

    let test =
      "Wednesday night: Chance of flurries. Temperature rising to minus 2 by morning. POP 40%";
    let (_, forecast) = parse_forecast::<Celsius>(test).unwrap();

    assert!(matches!(
      forecast,
      Forecast {
        celsius: Temperature::High(n, _),
        fahrenheit: Temperature::High(..),
        description,
        day: DayNight::Night,
        day_of_week: DayOfWeek::Wednesday,
      } if n == -2.
    ));
  }

  #[test]
  fn test_parse_positive() {
    let test = "Thursday: Snow. High plus 2.";
    let (_, forecast) = parse_forecast::<Celsius>(test).unwrap();

    assert!(matches!(
      forecast,
      Forecast {
        celsius: Temperature::High(n, PhantomData),
        fahrenheit: Temperature::High(_, PhantomData),
        description,
        day: DayNight::Day,
        day_of_week: DayOfWeek::Thursday,
      } if n == 2. && description == "Snow."
    ));
  }

  #[test]
  fn test_parse_current() {
    let test = "Current Conditions: Light Snow, -3.4°C";

    let (_, result) = parse_current_forecast(test).unwrap();

    assert!(
      matches!(result, CurrentForecast { celsius: Temperature::Current(n, _), description, .. } if n == -3.4 && description == "Light Snow")
    );
  }
}
