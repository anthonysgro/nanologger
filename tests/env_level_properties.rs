use nanologger::{LogLevel, LoggerBuilder};
use proptest::prelude::*;
use serial_test::serial;

fn arb_log_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::Error),
        Just(LogLevel::Warn),
        Just(LogLevel::Info),
        Just(LogLevel::Debug),
        Just(LogLevel::Trace),
    ]
}

/// Randomize the case of each character in a string.
fn randomize_case(s: &str, seed: &[bool]) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if seed.get(i).copied().unwrap_or(false) {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

proptest! {
    /// NANOLOGGER_LEVEL round-trips through any case variation.
    #[test]
    #[serial]
    fn env_var_round_trip_case_insensitive(
        level in arb_log_level(),
        case_bits in prop::collection::vec(any::<bool>(), 5..=5)
    ) {
        let level_str = level.to_string();
        let randomized = randomize_case(&level_str, &case_bits);
        std::env::set_var("NANOLOGGER_LEVEL", &randomized);
        let builder = LoggerBuilder::new();
        std::env::remove_var("NANOLOGGER_LEVEL");
        prop_assert_eq!(builder.get_level(), level);
    }

    /// Invalid NANOLOGGER_LEVEL values fall back to Info.
    #[test]
    #[serial]
    fn invalid_env_var_falls_back_to_info(s in "[a-zA-Z0-9_]{1,20}") {
        let valid = ["error", "warn", "info", "debug", "trace"];
        prop_assume!(!valid.contains(&s.to_ascii_lowercase().as_str()));
        std::env::set_var("NANOLOGGER_LEVEL", &s);
        let builder = LoggerBuilder::new();
        std::env::remove_var("NANOLOGGER_LEVEL");
        prop_assert_eq!(builder.get_level(), LogLevel::Info);
    }

    /// Explicit .level() always overrides the env var.
    #[test]
    #[serial]
    fn explicit_level_overrides_env_var(
        env_level in arb_log_level(),
        explicit_level in arb_log_level()
    ) {
        std::env::set_var("NANOLOGGER_LEVEL", env_level.to_string());
        let builder = LoggerBuilder::new().level(explicit_level);
        std::env::remove_var("NANOLOGGER_LEVEL");
        prop_assert_eq!(builder.get_level(), explicit_level);
    }
}
