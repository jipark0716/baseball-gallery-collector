use super::layer::ClickhouseSpan;
use super::visitor::{ClickhouseVisitor, Extra};
use clickhouse::Row;
use entity::Entity;
use serde_with::serde_as;
use tracing::{instrument, Level, Metadata};
use uuid::Uuid;

#[serde_as]
#[derive(Debug, serde::Serialize, serde::Deserialize, Entity, Row)]
#[entity(table = "logs")]
pub(crate) struct LogEntity {
    #[serde(with = "clickhouse::serde::time::datetime64::micros")] timestamp: time::OffsetDateTime,
    #[serde(with = "clickhouse::serde::uuid")] uuid: Uuid,
    #[serde(with = "super::level_serializer")] level: Level,
    extra_keys: Vec<String>,
    extra_values: Vec<String>,
    causes: Option<String>,
    file: Option<String>,
    line: Option<u64>,
    module_path: Option<String>,
    target: Option<String>,
    message: String,
    #[serde(with = "clickhouse::serde::uuid::option")] user_id: Option<Uuid>,
}

impl TryFrom<(ClickhouseVisitor, ClickhouseSpan, &Metadata<'_>)> for LogEntity {
    type Error = anyhow::Error;

    #[instrument]
    fn try_from(v: (ClickhouseVisitor, ClickhouseSpan, &Metadata)) -> Result<Self, Self::Error> {
        let (visitor, span, meta) = v;

        let extras: Vec<Extra> = [
            visitor.extras,
            span.extras,
        ].concat();

        let (extra_keys, extra_values) = extras
            .into_iter()
            .map(|x| (x.0.to_string(), x.1))
            .unzip();

        Ok(Self {
            uuid: uuid::Uuid::now_v7(),
            timestamp: time::OffsetDateTime::now_utc(),
            level: meta.level().clone(),
            user_id: span.user_id,
            extra_keys,
            extra_values,
            causes: visitor.causes,
            file: visitor.file,
            line: visitor.line,
            module_path: visitor.module_path,
            target: visitor.target,
            message: visitor
                .message
                .ok_or_else(|| anyhow::anyhow!("message is empty"))?,
        })
    }
}
