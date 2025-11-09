use crate::config;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry_otlp::{Protocol, WithExportConfig};

pub(crate) async fn init() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    log_init()?;
    Ok(())
}

fn log_init() -> anyhow::Result<()> {
    let traces_exporter = config::get_env_rseata_traces_exporter();
    if traces_exporter == "OTLP" {
        let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .build()?;

        // Create a tracer provider with the exporter
        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(otlp_exporter)
            .build();

        // Set it as the global provider
        global::set_tracer_provider(tracer_provider);

        // Get a tracer and create spans
        let tracer = global::tracer("my_tracer");
        tracer.in_span("doing_work", |_cx| {
            // Your application logic here...
        });

        Ok(())
    } else {
        tracing_subscriber::fmt::init();
        Ok(())
    }
}
