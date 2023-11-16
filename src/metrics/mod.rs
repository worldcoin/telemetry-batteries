pub mod statsd;

pub trait MetricsBattery {
    fn init(&self);
}
