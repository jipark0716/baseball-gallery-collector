use entity_derive;
pub use entity_derive::Entity;

pub trait Entity: serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn table_name() -> &'static str;
}
