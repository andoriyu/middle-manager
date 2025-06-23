use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::Duration;

/// Module for custom serialization of FixedOffset
mod fixed_offset_serde {
    use chrono::FixedOffset;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(offset: &FixedOffset, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as seconds from UTC
        serializer.serialize_i32(offset.local_minus_utc())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FixedOffset, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = i32::deserialize(deserializer)?;
        FixedOffset::east_opt(seconds)
            .or_else(|| FixedOffset::west_opt(-seconds))
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid offset seconds: {}", seconds)))
    }
}

/// Supported value types for memory properties.
#[derive(Clone, Debug, PartialEq, JsonSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MemoryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Bytes(Vec<u8>),
    // Non-recursive list of strings to avoid self-reference
    List(Vec<String>),
    // Map variant for key-value pairs
    Map(HashMap<String, String>),
    #[schemars(
        with = "String",
        title = "Date",
        description = "Date in YYYY-MM-DD format"
    )]
    Date(NaiveDate),
    #[schemars(
        with = "String",
        title = "Time",
        description = "Time in HH:MM:SS format"
    )]
    Time(NaiveTime),
    OffsetTime {
        #[schemars(
            with = "String",
            title = "Time with Offset",
            description = "Time in HH:MM:SS format"
        )]
        time: NaiveTime,
        #[schemars(
            with = "String",
            title = "UTC Offset",
            description = "Timezone offset in seconds from UTC"
        )]
        #[serde(with = "fixed_offset_serde")]
        offset: FixedOffset,
    },
    #[schemars(
        with = "String",
        title = "DateTime",
        description = "Date and time with timezone in RFC 3339 format"
    )]
    DateTime(DateTime<FixedOffset>),
    #[schemars(
        with = "String",
        title = "Local DateTime",
        description = "Date and time without timezone"
    )]
    LocalDateTime(NaiveDateTime),
    #[schemars(
        with = "String",
        title = "Duration",
        description = "Duration in nanoseconds"
    )]
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
                serde_json::Value::Array(l.into_iter().map(serde_json::Value::String).collect())
            },
            MemoryValue::Map(m) => {
                let mut map = serde_json::Map::new();
                for (k, v) in m {
                    map.insert(k, serde_json::Value::String(v));
                }
                serde_json::Value::Object(map)
            },
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
            serde_json::Value::Array(arr) => {
                // Convert array to list of strings
                let strings = arr.into_iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s,
                        _ => v.to_string(),
                    })
                    .collect();
                MemoryValue::List(strings)
            },
            serde_json::Value::Object(obj) => {
                // Convert object to map of strings
                let mut map = HashMap::new();
                for (k, v) in obj {
                    let value_str = match v {
                        serde_json::Value::String(s) => s,
                        _ => v.to_string(),
                    };
                    map.insert(k, value_str);
                }
                MemoryValue::Map(map)
            },
            serde_json::Value::Null => MemoryValue::String("null".to_string()),
        })
    }
}
impl std::fmt::Display for MemoryValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryValue::String(s) => write!(f, "{}", s),
            MemoryValue::Integer(i) => write!(f, "{}", i),
            MemoryValue::Float(fl) => write!(f, "{}", fl),
            MemoryValue::Boolean(b) => write!(f, "{}", b),
            MemoryValue::Bytes(bytes) => write!(f, "{:?}", bytes),
            MemoryValue::List(items) => write!(f, "{:?}", items),
            MemoryValue::Map(map) => write!(f, "{:?}", map),
            MemoryValue::Date(d) => write!(f, "{}", d),
            MemoryValue::Time(t) => write!(f, "{}", t),
            MemoryValue::OffsetTime { time, offset } => write!(f, "{}+{}", time, offset),
            MemoryValue::DateTime(dt) => write!(f, "{}", dt),
            MemoryValue::LocalDateTime(dt) => write!(f, "{}", dt),
            MemoryValue::Duration(d) => write!(f, "{:?}", d),
        }
    }
}
