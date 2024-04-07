//! Database record ID type.

use serde::{de, Deserialize, Deserializer, Serialize};
use surrealdb::sql::{self, Thing};

/// Type to deserialize SurrealDB Thing and serialize it as string or integer.
#[derive(Serialize)]
pub struct Id<T: Serialize>(T);

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
