use serde::{de, Deserialize};
use std::str::FromStr;
use tracing::Level;

pub fn serialize<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(level.as_str())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Level::from_str(&s).map_err(de::Error::custom)
}