use std::cell::RefCell;

use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry_sdk::trace::IdGenerator;
use rand::prelude::*;
use rand::rngs;

/// Reduced Id Generator
///
/// Generates trace ids using only 64 bits of randomness to be compatible
/// with other languages.
#[derive(Debug)]
pub struct ReducedIdGenerator;

impl IdGenerator for ReducedIdGenerator {
    fn new_trace_id(&self) -> TraceId {
        CURRENT_RNG.with(|rng| {
            let trace_id = rng.borrow_mut().random::<u64>();

            TraceId::from(trace_id as u128)
        })
    }

    fn new_span_id(&self) -> SpanId {
        CURRENT_RNG.with(|rng| SpanId::from(rng.borrow_mut().random::<u64>()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}
