use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;

/// Supported value types for memory properties.
#[derive(Clone, Debug, PartialEq, JsonSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MemoryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Bytes(Vec<u8>),
    List(Vec<MemoryValue>),
    #[schemars(with = "String")]
    Date(NaiveDate),
    #[schemars(with = "String")]
    Time(NaiveTime),
    OffsetTime {
        #[schemars(with = "String")]
        time: NaiveTime,
        #[schemars(with = "String")]
        offset: FixedOffset,
    },
    #[schemars(with = "String")]
    DateTime(DateTime<FixedOffset>),
    #[schemars(with = "String")]
    LocalDateTime(NaiveDateTime),
    #[schemars(with = "String")]
    Duration(Duration),
}

impl From<MemoryValue> for serde_json::Value {
    fn from(v: MemoryValue) -> Self {
        match v {
            MemoryValue::String(s) => serde_json::Value::String(s),
            MemoryValue::Integer(i) => serde_json::Value::Number(i.into()),
            MemoryValue::Float(f) => serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            MemoryValue::Boolean(b) => serde_json::Value::Bool(b),
            MemoryValue::Bytes(b) => serde_json::Value::Array(
                b.into_iter()
                    .map(|v| serde_json::Value::Number(v.into()))
                    .collect(),
            ),
            MemoryValue::List(l) => {
                serde_json::Value::Array(l.into_iter().map(Into::into).collect())
            }
            MemoryValue::Date(d) => serde_json::Value::String(d.to_string()),
            MemoryValue::Time(t) => serde_json::Value::String(t.to_string()),
            MemoryValue::OffsetTime { time, offset } => {
                serde_json::json!({"time": time.to_string(), "offset": offset.to_string()})
            }
            MemoryValue::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
            MemoryValue::LocalDateTime(dt) => serde_json::Value::String(dt.to_string()),
            MemoryValue::Duration(d) => serde_json::Value::String(format!("{}", d.as_nanos())),
        }
    }
}

impl TryFrom<serde_json::Value> for MemoryValue {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match value {
            serde_json::Value::String(s) => MemoryValue::String(s),
            serde_json::Value::Bool(b) => MemoryValue::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    MemoryValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    MemoryValue::Float(f)
                } else {
                    MemoryValue::String(n.to_string())
                }
            }
            serde_json::Value::Array(arr) => MemoryValue::List(
                arr.into_iter()
                    .map(MemoryValue::try_from)
                    .collect::<Result<_, _>>()?,
            ),
            serde_json::Value::Object(_) | serde_json::Value::Null => {
                MemoryValue::String(value.to_string())
            }
        })
    }
}
