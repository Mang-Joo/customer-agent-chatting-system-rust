use axum::{extract::Request, middleware::Next, response::Response};
use http::Version;
use opentelemetry::{global, trace::TraceContextExt};
use tracing::{field, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub async fn trace_middleware(request: Request, next: Next) -> Response {
    let http_method = request.method().to_string();
    let http_route = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // connection_info는 axum의 설정에 따라 다르게 가져올 수 있습니다
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let span = tracing::info_span!(
        "http_request",
        // HTTP 정보
        http.method = %http_method,
        // http.route = %http_route,
        http.flavor = %http_flavor(request.version()),
        http.scheme = "http",  // 또는 설정에 따라 "https"
        http.host = %host,
        http.client_ip = %client_ip,
        http.user_agent = %user_agent,
        http.target = %request.uri().path_and_query().map(|p| p.as_str()).unwrap_or(""),
        http.status_code = field::Empty,

        // OpenTelemetry 메타데이터
        otel.name = %format!("{} {}", http_method, http_route),
        otel.kind = "server",
        otel.status_code = field::Empty,

        // 에러 정보
        error = field::Empty,
    );
    let extractor = opentelemetry_http::HeaderExtractor(request.headers());
    let remote_context =
        global::get_text_map_propagator(|propagator| propagator.extract(&extractor));
    span.set_parent(remote_context);
    let context = span.context();

    let span_id = context.span().span_context().span_id();
    let trace_id = context.span().span_context().trace_id();

    span.record("transaction.id", span_id.to_string());
    span.record("trace.id", trace_id.to_string());

    let response = next.run(request).instrument(span.clone()).await;

    // Status code 기록
    let status = response.status().as_u16();
    span.record("http.status_code", &status);

    // OpenTelemetry status 설정
    let otel_status = match status {
        200..=299 => "OK",
        _ => "ERROR",
    };
    span.record("otel.status_code", &otel_status);

    response
}
fn http_flavor(version: Version) -> &'static str {
    match version {
        Version::HTTP_09 => "0.9",
        Version::HTTP_10 => "1.0",
        Version::HTTP_11 => "1.1",
        Version::HTTP_2 => "2.0",
        Version::HTTP_3 => "3.0",
        _ => "unknown",
    }
}
