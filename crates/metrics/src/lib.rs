//! Prometheus metrics integration for Sentinel.

use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, Encoder,
    GaugeVec, HistogramVec, TextEncoder,
};

/// Counter for HTTP requests per endpoint, method, and status code.
pub static HTTP_REQUESTS: std::sync::LazyLock<CounterVec> = std::sync::LazyLock::new(|| {
    register_counter_vec!(
        "sentinel_http_requests_total",
        "Total HTTP requests handled by Sentinel",
        &["method", "path", "status"]
    )
    .expect("Failed to register HTTP request counter")
});

/// Histogram for HTTP request duration in seconds.
pub static HTTP_REQUEST_DURATION: std::sync::LazyLock<HistogramVec> =
    std::sync::LazyLock::new(|| {
        register_histogram_vec!(
            "sentinel_http_request_duration_seconds",
            "HTTP request duration in seconds",
            &["method", "path"]
        )
        .expect("Failed to register HTTP request duration histogram")
    });

/// Gauge for number of active workflows per worker.
pub static ACTIVE_WORKFLOWS: std::sync::LazyLock<GaugeVec> = std::sync::LazyLock::new(|| {
    register_gauge_vec!(
        "sentinel_active_workflows",
        "Number of workflows currently being processed by a worker",
        &["worker_id"]
    )
    .expect("Failed to register active workflow gauge")
});

/// Counter for workflow state transitions.
pub static WORKFLOW_STATE_TRANSITIONS: std::sync::LazyLock<CounterVec> =
    std::sync::LazyLock::new(|| {
        register_counter_vec!(
            "sentinel_workflow_state_transitions_total",
            "Total workflow state transitions",
            &["from_state", "to_state", "worker_id"]
        )
        .expect("Failed to register workflow state transitions counter")
    });

/// Counter for policy evaluation decisions.
pub static POLICY_DECISIONS: std::sync::LazyLock<CounterVec> = std::sync::LazyLock::new(|| {
    register_counter_vec!(
        "sentinel_policy_decisions_total",
        "Total policy evaluation decisions",
        &["decision"]
    )
    .expect("Failed to register policy decisions counter")
});

/// Counter for attestation operations.
pub static ATTESTATION_OPERATIONS: std::sync::LazyLock<CounterVec> =
    std::sync::LazyLock::new(|| {
        register_counter_vec!(
            "sentinel_attestation_operations_total",
            "Total attestation operations",
            &["operation", "result"]
        )
        .expect("Failed to register attestation operations counter")
    });

/// Export all metrics as a Prometheus exposition string.
pub fn gather_metrics() -> Result<String, prometheus::Error> {
    let mut buffer = Vec::new();
    let metric_families = prometheus::gather();
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer).expect("Metrics not UTF-8"))
}

/// Middleware layer for Axum to record HTTP metrics.
pub fn metrics_layer() -> tower::layer::LayerFn<fn(axum::Router) -> axum::Router> {
    tower::layer::layer_fn(|router: axum::Router| {
        router.layer(axum::middleware::from_fn(metrics_middleware))
    })
}

async fn metrics_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl axum::response::IntoResponse {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16().to_string();
    let duration = start.elapsed().as_secs_f64();

    HTTP_REQUESTS
        .with_label_values(&[&method, &path, &status])
        .inc();

    HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &path])
        .observe(duration);

    response
}
