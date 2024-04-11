//! Database record ID type.

use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use surrealdb::sql::{self, Thing};
use utoipa::{openapi::{ObjectBuilder, RefOr, Schema, SchemaType}, ToSchema};

/// Type to deserialize SurrealDB Thing and serialize it as string or integer.
#[derive(Debug, Serialize)]
pub struct Id<T: Serialize>(pub T);

impl<'de> Deserialize<'de> for Id<String> {
    fn deserialize<D>(
        deserializer: D
    ) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match Thing::deserialize(deserializer)?.id {
            sql::Id::String(id) => Ok(Self(id)),
            _ => Err(de::Error::custom("record id is not a string")),
        }
    }
}

impl<'de> Deserialize<'de> for Id<i64> {
    fn deserialize<D>(
        deserializer: D
    ) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match Thing::deserialize(deserializer)?.id {
            sql::Id::Number(id) => Ok(Self(id)),
            _ => Err(de::Error::custom("record id is not an integer")),
        }
    }
}

impl<'r> ToSchema<'r> for Id<String> {
    fn schema() -> (&'r str, RefOr<Schema>) {
        (
            "Id",
            ObjectBuilder::new()
                .schema_type(SchemaType::String)
                .example(Some(Value::String("3o8uj9c2q11i4pu0zgbc".into())))
                .into()
        )
    }
}

impl<'r> ToSchema<'r> for Id<i64> {
    fn schema() -> (&'r str, RefOr<Schema>) {
        (
            "Id",
            ObjectBuilder::new()
                .schema_type(SchemaType::Number)
                .example(Some(Value::Number(42.into())))
                .into()
        )
    }
}
