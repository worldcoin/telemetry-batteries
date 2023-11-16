pub mod metrics;
pub mod tracing;

#[derive(Default)]
pub struct TelemetryBatteries {
    pub batteries: Vec<Box<dyn TelemetryBattery>>,
}

impl TelemetryBatteries {
    pub fn new() -> Self {
        Self::default()
    }

    //TODO: do we need static here?
    pub fn add_battery<T: TelemetryBattery + 'static>(&mut self, battery: T) {
        self.batteries.push(Box::new(battery));
    }

    pub fn init(self) {
        for battery in self.batteries.iter() {
            battery.init();
        }
    }
}

impl Drop for TelemetryBatteries {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub trait TelemetryBattery {
    fn init(&self);
}
