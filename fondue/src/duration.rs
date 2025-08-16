use std::convert::TryFrom;
use std::time::Duration;
use thiserror::Error;

/// Errors encountered when parsing duration strings
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DurationParseError {
    #[error("empty duration string")]
    EmptyString,

    #[error("no time unit found in duration string")]
    MissingUnit,

    #[error("invalid number '{0}'")]
    InvalidNumber(String),

    #[error("unknown time unit '{0}'")]
    UnknownUnit(String),
}

/// Supported time units for duration parsing
#[derive(Debug, PartialEq, Eq)]
enum TimeUnit {
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
}

impl TryFrom<&str> for TimeUnit {
    type Error = DurationParseError;

    /// Converts a string representation of a unit into `TimeUnit`
    fn try_from(unit: &str) -> Result<Self, Self::Error> {
        match unit.to_lowercase().as_str() {
            "ns" | "nanosecond" | "nanoseconds" => Ok(TimeUnit::Nanosecond),
            "us" | "µs" | "microsecond" | "microseconds" => Ok(TimeUnit::Microsecond),
            "ms" | "millisecond" | "milliseconds" => Ok(TimeUnit::Millisecond),
            "s" | "sec" | "second" | "seconds" => Ok(TimeUnit::Second),
            "m" | "min" | "minute" | "minutes" => Ok(TimeUnit::Minute),
            "h" | "hr" | "hour" | "hours" => Ok(TimeUnit::Hour),
            "d" | "day" | "days" => Ok(TimeUnit::Day),
            unknown => Err(DurationParseError::UnknownUnit(unknown.to_string())),
        }
    }
}

/// Parses duration strings like "1.5h", "200ms", "30s", supporting fractional values.
/// Returns a `Duration` or a detailed parsing error.
///
/// # Errors
/// Returns variants of `DurationParseError` if input is empty, missing unit,
/// contains an invalid number, or an unknown unit.
pub fn parse_duration(s: &str) -> Result<Duration, DurationParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(DurationParseError::EmptyString);
    }

    // Find the first alphabetic character to split number and unit
    let pos = s
        .find(|c: char| c.is_alphabetic())
        .ok_or(DurationParseError::MissingUnit)?;

    let (num_str, unit_str) = s.split_at(pos);
    let num_str = num_str.trim();
    let unit_str = unit_str.trim();

    // Reject negative numbers; durations can't be negative
    if num_str.starts_with('-') {
        return Err(DurationParseError::InvalidNumber(num_str.to_string()));
    }

    // Parse the number as f64 for fractional support
    let number: f64 = num_str
        .parse()
        .map_err(|_| DurationParseError::InvalidNumber(num_str.to_string()))?;

    let unit = TimeUnit::try_from(unit_str)?;

    // Convert number and unit into std::time::Duration
    let duration = match unit {
        TimeUnit::Nanosecond => Duration::from_nanos(number.round() as u64),
        TimeUnit::Microsecond => Duration::from_micros(number.round() as u64),
        TimeUnit::Millisecond => Duration::from_millis(number.round() as u64),
        TimeUnit::Second => Duration::from_secs_f64(number),
        TimeUnit::Minute => Duration::from_secs_f64(number * 60.0),
        TimeUnit::Hour => Duration::from_secs_f64(number * 3600.0),
        TimeUnit::Day => Duration::from_secs_f64(number * 86400.0),
    };

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_basic_units() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
        assert_eq!(parse_duration("200ms").unwrap(), Duration::from_millis(200));
        assert_eq!(parse_duration("500us").unwrap(), Duration::from_micros(500));
        assert_eq!(parse_duration("100ns").unwrap(), Duration::from_nanos(100));
    }

    #[test]
    fn test_fractional_values() {
        assert_eq!(parse_duration("1.5s").unwrap(), Duration::from_millis(1500));
        assert_eq!(parse_duration("0.5m").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("1.25h").unwrap(), Duration::from_secs(4500));
    }

    #[test]
    fn test_case_and_whitespace() {
        assert_eq!(parse_duration(" 10 S ").unwrap(), Duration::from_secs(10));
        assert_eq!(
            parse_duration("  2   hr ").unwrap(),
            Duration::from_secs(7200)
        );
    }

    #[test]
    fn test_invalid_inputs() {
        assert_eq!(
            parse_duration("").unwrap_err(),
            DurationParseError::EmptyString
        );
        assert_eq!(
            parse_duration("ms").unwrap_err(),
            DurationParseError::MissingUnit
        );
        assert_eq!(
            parse_duration("abc").unwrap_err(),
            DurationParseError::MissingUnit
        );
        assert_eq!(
            parse_duration("100xy").unwrap_err(),
            DurationParseError::UnknownUnit("xy".to_string())
        );
        assert_eq!(
            parse_duration("100").unwrap_err(),
            DurationParseError::MissingUnit
        );
        assert_eq!(
            parse_duration("-10s").unwrap_err(),
            DurationParseError::InvalidNumber("-10".to_string())
        );
    }
}
