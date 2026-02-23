use nanologger::LogLevel;
use proptest::prelude::*;
use std::str::FromStr;

fn arb_log_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::Error),
        Just(LogLevel::Warn),
        Just(LogLevel::Info),
        Just(LogLevel::Debug),
        Just(LogLevel::Trace),
    ]
}

proptest! {
    /// LogLevel ordering is total and consistent.
    #[test]
    fn test_loglevel_ordering(a in arb_log_level(), b in arb_log_level()) {
        // Severity hierarchy: Error < Warn < Info < Debug < Trace
        let expected = [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];
        let pos_a = expected.iter().position(|&x| x == a).unwrap();
        let pos_b = expected.iter().position(|&x| x == b).unwrap();
        prop_assert_eq!(a.cmp(&b), pos_a.cmp(&pos_b));
    }

    /// LogLevel string round-trip: parse(display(level)) == level.
    #[test]
    fn test_loglevel_roundtrip(level in arb_log_level()) {
        let s = level.to_string();
        let parsed = LogLevel::from_str(&s).unwrap();
        prop_assert_eq!(level, parsed);
    }

    /// Non-level strings are rejected by from_str.
    #[test]
    fn test_invalid_level_rejected(s in "[a-zA-Z0-9_]{1,20}") {
        let valid = ["error", "warn", "info", "debug", "trace"];
        if !valid.contains(&s.to_ascii_lowercase().as_str()) {
            prop_assert!(LogLevel::from_str(&s).is_err());
        }
    }

    /// LogLevel u8 round-trip: from_u8(as_u8(level)) == Some(level).

    #[test]
    fn test_loglevel_u8_round_trip(level in arb_log_level()) {
        let val = level.as_u8();
        prop_assert_eq!(LogLevel::from_u8(val), Some(level));
    }

    #[test]
    fn test_invalid_u8_returns_none(val in 5u8..=u8::MAX) {
        prop_assert_eq!(LogLevel::from_u8(val), None);
    }
}
