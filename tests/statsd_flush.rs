//! The StatsD backend tags metrics and flushes them on an interval, without waiting for the
//! buffer to fill. Its own process: the global recorder can be installed only once.
#![cfg(feature = "metrics-statsd")]

use std::{net::UdpSocket, time::Duration};

use telemetry_batteries::{
    MetricsBackend, MetricsConfig, StatsdConfig, TelemetryConfig,
    TelemetryPreset,
};

#[test]
fn tagged_metrics_are_flushed_on_the_interval_and_on_shutdown() {
    let agent = UdpSocket::bind("127.0.0.1:0").unwrap();
    agent
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let config = TelemetryConfig {
        preset: TelemetryPreset::None,
        metrics: MetricsConfig {
            backend: MetricsBackend::Statsd,
            statsd: StatsdConfig {
                host: "127.0.0.1".to_owned(),
                port: agent.local_addr().unwrap().port(),
                default_tags: vec![
                    ("service".to_owned(), "svc".to_owned()),
                    ("dd.internal.entity_id".to_owned(), "pod-uid".to_owned()),
                ],
                flush_interval: Duration::from_millis(100),
                ..StatsdConfig::default()
            },
            ..MetricsConfig::default()
        },
        ..TelemetryConfig::default()
    };
    let guard = telemetry_batteries::init_with_config(config).unwrap();
    let mut packet = [0; 1024];

    // Far below the 1 KiB buffer, so only the flush thread can send it.
    metrics::counter!("requests").increment(1);
    let len = agent.recv(&mut packet).expect("flushed within the timeout");
    let received = std::str::from_utf8(&packet[..len]).unwrap();
    assert!(received.starts_with("requests:1|c|#"), "{received}");
    assert!(received.contains("service:svc"), "{received}");
    assert!(
        received.contains("dd.internal.entity_id:pod-uid"),
        "{received}"
    );

    // Recorded just before shutdown: the guard's final flush sends it.
    metrics::counter!("last").increment(1);
    std::thread::sleep(Duration::from_millis(100)); // let the queue worker buffer it
    drop(guard);
    let len = agent.recv(&mut packet).expect("flushed on shutdown");
    let received = std::str::from_utf8(&packet[..len]).unwrap();
    assert!(received.contains("last:1|c"), "{received}");
}
