use serde::{de, Deserialize, Deserializer};
use uuid::Uuid;

pub fn deserialize_optional_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    Ok(s.and_then(|s| {
        if s.to_lowercase() == "null" {
            None
        } else {
            Some(Uuid::parse_str(&s).map_err(de::Error::custom))
        }
    })
    .transpose()?)
}
