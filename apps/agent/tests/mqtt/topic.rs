// internal crates
use config_agent::mqtt::topics;


mod parse_subscription {
    use super::*;

    #[test]
    fn test_device_sync() {
        let topic = topics::device_sync("123");
        assert!(topics::parse_subscription("123", &topic) == topics::SubscriptionTopics::Sync);
    }

    #[test]
    fn test_device_ping() {
        let topic = topics::device_ping("123");
        assert!(topics::parse_subscription("123", &topic) == topics::SubscriptionTopics::Ping);
    }

    #[test]
    fn test_device_unknown() {

        let unknown_topics = vec![
            "v1/cmd/devices/123/unknown",
            "v2/cmd/devices/123/ping",
            "arglechargle",
        ];
        for topic in unknown_topics {
            assert!(topics::parse_subscription("123", topic) == topics::SubscriptionTopics::Unknown);
        }
    }

}