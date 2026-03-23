use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_sdk::{
    Resource,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use opentelemetry_semantic_conventions::{
    SCHEMA_URL,
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_VERSION},
};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn resource() -> Resource {
    Resource::builder()
        .with_service_name(env!("CARGO_PKG_NAME"))
        .with_schema_url(
            [
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
            ],
            SCHEMA_URL,
        )
        .build()
}

fn init_tracer_provider() -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource())
        .with_batch_exporter(exporter)
        .build()
}

pub fn init_tracing_subscriber() -> OtelGuard {
    let tracer_provider = init_tracer_provider();

    let tracer = tracer_provider.tracer("tracing-otel-subscriber");

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=debug"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .with(env_filter)
        .init();

    OtelGuard { tracer_provider }
}

pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("{err:?}");
        }
    }
}
