//! Gauge increments and decrements reach StatsD as absolute values. Its own process: the
//! global recorder can be installed only once.
#![cfg(feature = "metrics-statsd")]

use std::{net::UdpSocket, time::Duration};

use telemetry_batteries::{
    MetricsBackend, MetricsConfig, StatsdConfig, TelemetryConfig,
    TelemetryPreset,
};

#[test]
fn relative_gauge_updates_are_sent_as_the_current_value() {
    let agent = UdpSocket::bind("127.0.0.1:0").unwrap();
    agent
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    let config = TelemetryConfig {
        preset: TelemetryPreset::None,
        metrics: MetricsConfig {
            backend: MetricsBackend::Statsd,
            statsd: StatsdConfig {
                host: "127.0.0.1".to_owned(),
                port: agent.local_addr().unwrap().port(),
                flush_interval: Duration::from_millis(50),
                ..StatsdConfig::default()
            },
            ..MetricsConfig::default()
        },
        ..TelemetryConfig::default()
    };
    let guard = telemetry_batteries::init_with_config(config).unwrap();

    // Separate handles for the same key share one value, as `gauge!` at different call sites do.
    metrics::gauge!("active").increment(2.0);
    metrics::gauge!("active").decrement(1.0);
    metrics::gauge!("active").increment(1.0);
    metrics::gauge!("active", "pool" => "b").increment(5.0);
    drop(guard);

    let mut lines = Vec::new();
    let mut packet = [0; 1024];
    while let Ok(len) = agent.recv(&mut packet) {
        let text = std::str::from_utf8(&packet[..len]).unwrap().to_owned();
        lines.extend(text.lines().map(str::to_owned));
    }
    let active: Vec<_> = lines
        .iter()
        .filter(|line| line.starts_with("active:"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        active,
        [
            "active:2|g",
            "active:1|g",
            "active:2|g",
            "active:5|g|#pool:b"
        ],
        "{lines:?}"
    );
}
