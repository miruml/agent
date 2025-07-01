use config_agent::logs::LogLevel;
use std::collections::HashSet;

#[test]
fn deserialize_log_level() {
    struct TestCase {
        input: &'static str,
        expected: LogLevel,
    }

    let test_cases = vec![
        TestCase {
            input: "\"trace\"",
            expected: LogLevel::Trace,
        },
        TestCase {
            input: "\"debug\"",
            expected: LogLevel::Debug,
        },
        TestCase {
            input: "\"info\"",
            expected: LogLevel::Info,
        },
        TestCase {
            input: "\"warn\"",
            expected: LogLevel::Warn,
        },
        TestCase {
            input: "\"error\"",
            expected: LogLevel::Error,
        },
    ];

    let mut variants = LogLevel::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<LogLevel>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
}
