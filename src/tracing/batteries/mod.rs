use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, Layer, Registry,
};

pub mod datadog;

pub struct TracingBattery;

impl TracingBattery {
    pub fn init<L>(layers: Option<L>)
    where
        L: Layer<Registry> + Send + Sync,
    {
        tracing_subscriber::registry().with(layers).init();
    }
}
