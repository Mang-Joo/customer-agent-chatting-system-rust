use trace::{setup_subscribers, setup_trace_provider};
use tracing_appender::non_blocking::WorkerGuard;

mod formatter;
pub mod middleware;
pub mod trace;

pub fn init_telemetry() -> WorkerGuard {
    let trace_provider = setup_trace_provider();
    let guard = setup_subscribers(trace_provider);
    guard
}
