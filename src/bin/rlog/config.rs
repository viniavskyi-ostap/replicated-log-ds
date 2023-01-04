pub const SECONDARY_URLS: [&'static str; 2] = ["localhost:8081", "localhost:8082"];
// timeout for requests to secondaries from master in seconds
pub const SECONDARY_REQUEST_TIMEOUT: u64 = 15;
// master retry interval in seconds
pub const RETRY_INTERVAL: u64 = 5;
// heartbeat check interval in seconds
pub const HEARTBEAT_CHECK_INTERVAL: u64 = 5;

// secondary sleep duration choices
pub const SECONDARY_SLEEP_DURATIONS: [u64; 3] = [1, 5, 10];

