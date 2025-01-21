use serde_json::{Map, Value};
use tracing_subscriber::{
    field::VisitOutput,
    fmt::{
        format::{JsonVisitor, Writer},
        FormatEvent, FormatFields,
    },
    registry::LookupSpan,
};

pub struct CustomJsonFormatter;

impl<S, N> FormatEvent<S, N> for CustomJsonFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut json = self.create_base_json(event);
        self.add_event_fields(&mut json, event)?;
        self.add_span_context(&mut json, ctx);
        self.add_metadata(&mut json);
        self.write_json(&mut writer, json)
    }
}

impl CustomJsonFormatter {
    fn create_base_json(&self, event: &tracing::Event<'_>) -> serde_json::Map<String, Value> {
        let mut json = serde_json::Map::new();
        json.insert(
            "level".to_string(),
            Value::String(event.metadata().level().to_string()),
        );
        json.insert(
            "target".to_string(),
            Value::String(event.metadata().target().to_string()),
        );
        json
    }

    fn add_event_fields(
        &self,
        json: &mut serde_json::Map<String, Value>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut visit = String::new();
        let mut visitor = JsonVisitor::new(&mut visit);
        event.record(&mut visitor);
        visitor.finish().map_err(|_| std::fmt::Error)?;

        let field_map: Map<String, Value> =
            serde_json::from_str(&visit).map_err(|_| std::fmt::Error)?;

        if let Some(message) = field_map.get("message") {
            json.insert("message".to_string(), message.clone());
        }

        Ok(())
    }

    fn add_span_context<S, N>(
        &self,
        json: &mut serde_json::Map<String, Value>,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
    ) where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
        N: for<'writer> FormatFields<'writer> + 'static,
    {
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                json.insert(
                    "span_name".to_string(),
                    Value::String(span.name().to_string()),
                );

                if let Some(fields) = span
                    .extensions()
                    .get::<tracing_subscriber::fmt::FormattedFields<N>>()
                {
                    if let Ok(Value::Object(fields_map)) =
                        serde_json::from_str::<Value>(&fields.fields)
                    {
                        json.extend(fields_map);
                    }
                }
            }
        }
    }

    fn add_metadata(&self, json: &mut serde_json::Map<String, Value>) {
        json.insert(
            "timestamp".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
        json.insert(
            "threadId".to_string(),
            Value::String(format!("{:?}", std::thread::current().id())),
        );
    }

    fn write_json(
        &self,
        writer: &mut Writer<'_>,
        json: serde_json::Map<String, Value>,
    ) -> std::fmt::Result {
        let json_string =
            serde_json::to_string(&Value::Object(json)).map_err(|_| std::fmt::Error)?;
        writeln!(writer, "{}", json_string)
    }
}
