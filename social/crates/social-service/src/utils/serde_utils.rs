
use rust_decimal::Decimal;
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

pub fn deserialize_null_as_zero<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<Decimal> = Deserialize::deserialize(deserializer).expect("Fuck");
    Ok(s.unwrap_or(Decimal::ZERO))
}
