use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime::Tokio};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::{self};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::config::RESOURCE;
use crate::constants::APP_NAME;

use super::formatter::CustomJsonFormatter;

pub fn setup_trace_provider() -> opentelemetry_sdk::trace::TracerProvider {
    let resource = RESOURCE.clone();
    global::set_text_map_propagator(TraceContextPropagator::default());

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()
        .expect("Failed to build the span exporter");

    opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(otlp_exporter, Tokio)
        .with_resource(resource)
        .build()
}

pub fn setup_subscribers(provider: opentelemetry_sdk::trace::TracerProvider) -> WorkerGuard {
    let tracer = provider.tracer(APP_NAME);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

    let (non_blocking, guard) = create_file_appender();

    // 기본 레지스트리 생성
    let subscriber = Registry::default();

    // 환경 필터 레이어 추가
    let subscriber = subscriber.with(env_filter);

    // OpenTelemetry 레이어 추가
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = subscriber.with(telemetry);

    // JSON 로깅 레이어 추가
    let fmt_layer = fmt::layer()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_level(true)
        .with_span_list(true)
        .with_current_span(true)
        .flatten_event(true)
        .with_writer(move || non_blocking.clone())
        .event_format(CustomJsonFormatter);

    let subscriber = subscriber.with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    guard
}

fn create_file_appender() -> (NonBlocking, WorkerGuard) {
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("application")
        .filename_suffix("log")
        .build("")
        .expect("failed to create RollingFileAppender");

    tracing_appender::non_blocking(file_appender)
}
