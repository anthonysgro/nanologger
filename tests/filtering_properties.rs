use proptest::prelude::*;
use nanologger::LogLevel;

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
    /// Level filtering emits if message severity >= filter severity.

    #[test]
    fn test_level_filtering(filter in arb_log_level(), message in arb_log_level()) {
        // The Logger emits when message_level <= filter_level (higher or equal severity).
        // Since LogLevel derives Ord with Error=0 < Warn=1 < Info=2 < Debug=3 < Trace=4,
        // a message with lower numeric value has higher severity.
        let should_emit = message <= filter;

        // Verify the invariant matches the severity hierarchy:
        // If filter is Info (2), messages Error(0), Warn(1), Info(2) should emit,
        // but Debug(3) and Trace(4) should not.
        let all_levels = [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];
        let filter_idx = all_levels.iter().position(|&l| l == filter).unwrap();
        let message_idx = all_levels.iter().position(|&l| l == message).unwrap();

        // Message emits iff its index <= filter's index (i.e., severity >= filter severity)
        let expected_emit = message_idx <= filter_idx;
        prop_assert_eq!(
            should_emit, expected_emit,
            "filter={:?}, message={:?}: m<=f says {}, index check says {}",
            filter, message, should_emit, expected_emit
        );
    }
}
