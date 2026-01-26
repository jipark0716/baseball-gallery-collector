use super::consumer::LogMessage;
use crate::clickhouse::entity::LogEntity;
use crate::clickhouse::visitor::{ClickhouseVisitor, Extra};
use dashmap::DashMap;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::field::Field;
use tracing::span::Attributes;
use tracing::Event;
use tracing::Id;
use tracing::Subscriber;
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use util::shutdown::Shutdown;
use uuid::Uuid;

pub struct ClickhouseLayer {
    spans: DashMap<Id, HashMap<&'static str, SpanValue>>,
    sender: Sender<LogMessage>,
}

impl ClickhouseLayer {
    pub(super) fn new(sender: Sender<LogMessage>) -> Self {
        Self {
            spans: DashMap::new(),
            sender,
        }
    }
}

pub struct ClickhouseLayerShutdown {
    join_handle: JoinHandle<()>,
    sender: Sender<LogMessage>,
}

impl ClickhouseLayerShutdown {
    pub(super) fn new(join_handle: JoinHandle<()>, sender: Sender<LogMessage>) -> Self {
        Self { join_handle, sender }
    }
}

#[async_trait::async_trait]
impl Shutdown for ClickhouseLayerShutdown {
    async fn shutdown(self) {
        let Self {
            join_handle: handle,
            sender,
        } = self;

        if let Some(e) = sender.send(LogMessage::Shutdown).await.err() {
            println!("clickhouse log shutdown fail (chan close) {}", e);
        }

        if let Some(e) = handle.await.err() {
            println!("clickhouse log shutdown fail (close wait) {}", e);
        }
    }
}

impl<S> Layer<S> for ClickhouseLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let mut visitor = SpanData::new();
        attrs.record(&mut visitor);

        self.spans.insert(id.clone(), visitor.fields);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let meta = event.metadata();

        let mut visitor = ClickhouseVisitor::new();

        let mut cur = ctx.lookup_current();

        let mut struct_span = ClickhouseSpan::default();
        while let Some(span) = cur {
            if let Some(span_attributes) = self.spans.get(&span.id()) {
                for (k, v) in span_attributes.iter() {
                    match v {
                        SpanValue::I64(v) => {
                            struct_span.extra(k, v.to_string())
                        }
                        SpanValue::U64(v) => {
                            struct_span.extra(k, v.to_string())
                        }
                        SpanValue::Bool(v) => {
                            struct_span.extra(k, v.to_string())
                        }
                        SpanValue::Str(v) => {
                            struct_span.extra(k, v.to_string())
                        }
                        SpanValue::Debug(v) => {
                            struct_span.extra(k, v.to_string())
                        }
                        SpanValue::U128(v) => struct_span.user_id = Some(Uuid::from_u128(*v)),
                    }
                }
            }

            cur = span.parent();
        }

        event.record(&mut visitor);
        let entity: LogEntity = match (visitor, struct_span, meta).try_into() {
            Ok(entity) => entity,
            Err(e) => return println!("visit fail {}", e),
        };

        if let Err(e) = self.sender.try_send(entity.into()) {
            println!("clickhouse log send fail {:?}", e)
        }
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        self.spans.remove(&id);
    }
}

#[derive(Default, Debug)]
pub struct ClickhouseSpan {
    pub(super) user_id: Option<Uuid>,
    pub(super) extras: Vec<Extra>,
}

impl ClickhouseSpan {
    pub fn extra(&mut self, k: &'static str, v: String) {
        self.extras.push(Extra(k, v))
    }
}

#[derive(Debug)]
enum SpanValue {
    U128(u128),
    I64(i64),
    U64(u64),
    Bool(bool),
    Str(String),
    Debug(String),
}

struct SpanData {
    fields: HashMap<&'static str, SpanValue>,
}

impl SpanData {
    fn new() -> Self {
        Self { fields: HashMap::new() }
    }
}

impl Visit for SpanData {
    fn record_i64(&mut self, f: &Field, v: i64) {
        self.fields.insert(f.name(), SpanValue::I64(v));
    }
    fn record_u64(&mut self, f: &Field, v: u64) {
        self.fields.insert(f.name(), SpanValue::U64(v));
    }

    fn record_u128(&mut self, f: &Field, v: u128) {
        self.fields.insert(f.name(), SpanValue::U128(v));
    }

    fn record_bool(&mut self, f: &Field, v: bool) {
        self.fields.insert(f.name(), SpanValue::Bool(v));
    }

    fn record_str(&mut self, f: &Field, v: &str) {
        self.fields.insert(f.name(), SpanValue::Str(v.to_string()));
    }

    fn record_debug(&mut self, f: &Field, v: &dyn std::fmt::Debug) {
        self.fields.insert(f.name(), SpanValue::Debug(format!("{:?}", v)));
    }
}