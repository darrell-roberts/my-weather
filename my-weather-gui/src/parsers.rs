use crate::types::{DayNight, DayOfWeek, ForeCast, Temperature};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, space0, space1},
    combinator::{map, map_res, opt, recognize, value},
    sequence::{delimited, tuple},
    IResult,
};

/// Parse an optionally signed number.
fn parse_number(input: &str) -> IResult<&str, f32> {
    let fraction_parse = recognize(tuple((digit1, char('.'), digit1)));
    let negative = recognize(tuple((tag("minus"), space1)));
    let num_parse = delimited(
        space0,
        tuple((opt(negative), alt((tag("zero"), fraction_parse, digit1)))),
        space0,
    );
    let mut parser = map_res(num_parse, |(neg, n): (Option<_>, &str)| {
        if n == "zero" {
            Ok(0.)
        } else {
            n.parse::<f32>()
                .map(|num| if neg.is_some() { -num } else { num })
        }
    });
    parser(input)
}

fn parse_temp(input: &str) -> IResult<&str, Temperature> {
    let high_parser = map(
        delimited(
            alt((tag("High"), tag("Temperature steady near"))),
            parse_number,
            char('.'),
        ),
        Temperature::High,
    );

    let low_parser = map(
        delimited(tag("Low"), parse_number, char('.')),
        Temperature::Low,
    );

    alt((high_parser, low_parser))(input)
}

fn parse_title(input: &str) -> IResult<&str, &str> {
    let parse_tags = alt((
        take_until("Low"),
        take_until("High"),
        take_until("Temperature steady near"),
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

pub fn parse_full_title(input: &str) -> IResult<&str, ForeCast> {
    let (input, day_of_week) = parse_day_of_week(input)?;
    let (input, day_night) = parse_day_night(input)?;
    let parser = tuple((map(parse_title, String::from), parse_temp));
    let mut parser = map(parser, |(description, temp)| ForeCast {
        day: day_night,
        day_of_week,
        temp,
        description,
    });
    parser(input)
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_parse_entry(input: &str, expected: (String, Temperature)) {
        let (_, forecast) = parse_full_title(input).unwrap();
        assert_eq!(forecast.description, expected.0);
        assert_eq!(forecast.temp, expected.1);
    }

    #[test]
    fn test_parse_temp() {
        test_parse_entry(
            "Saturday: A mix of sun and cloud. Temperature steady near minus 1.",
            ("A mix of sun and cloud.".into(), Temperature::High(-1.)),
        );
        test_parse_entry(
            "Saturday night: A few clouds. Low minus 12.",
            ("A few clouds.".into(), Temperature::Low(-12.)),
        );
    }

    #[test]
    fn test_parse_number() {
        let test = "minus 1. Forecast issued 3:45 PM EST Friday 06 January 2023";
        let result = parse_number(test);
        assert!(result.is_ok())
    }

    #[test]
    fn test_parse_day_of_week() {
        let test = "Monday: Sunny. High zero";

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
        let (_, forecast) = parse_full_title(test).unwrap();

        assert!(matches!(
            forecast,
            ForeCast {
                temp: Temperature::High(n),
                description,
                day: DayNight::Day,
                day_of_week: DayOfWeek::Monday,
            } if n == 0. && description == "Sunny."
        ));

        let test = "Sunday night: Cloudy periods. Low minus 9.";
        let (_, forecast) = parse_full_title(test).unwrap();

        assert!(matches!(
            forecast,
            ForeCast {
                temp: Temperature::Low(n),
                description,
                day: DayNight::Night,
                day_of_week: DayOfWeek::Sunday
            } if n == -9. && description == "Cloudy periods."
        ))
    }
}
