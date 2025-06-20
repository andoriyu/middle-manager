use std::collections::HashMap;

use mm_memory::{MemoryError, MemoryValue};
use neo4rs::BoltType;
use time::format_description::well_known::Rfc3339;

/// Convert a [`MemoryValue`] directly into a [`neo4rs::BoltType`].
///
/// This avoids the indirection through `serde_json::Value` when
/// sending data to Neo4j.
pub(crate) fn memory_value_to_bolt(
    value: &MemoryValue,
) -> Result<BoltType, MemoryError<neo4rs::Error>> {
    Ok(match value {
        MemoryValue::String(s) => s.clone().into(),
        MemoryValue::Integer(i) => (*i).into(),
        MemoryValue::Float(f) => (*f).into(),
        MemoryValue::Boolean(b) => (*b).into(),
        MemoryValue::Bytes(bytes) => bytes.clone().into(),
        MemoryValue::List(items) => {
            let bolt_items: Vec<BoltType> = items
                .iter()
                .map(memory_value_to_bolt)
                .collect::<Result<_, _>>()?;
            bolt_items.into()
        }
        MemoryValue::Date(d) => d.to_string().into(),
        MemoryValue::Time(t) => t.to_string().into(),
        MemoryValue::OffsetTime { time, offset } => {
            let mut map: HashMap<String, BoltType> = HashMap::new();
            map.insert("time".to_string(), time.to_string().into());
            map.insert("offset".to_string(), offset.to_string().into());
            map.into()
        }
        MemoryValue::DateTime(dt) => dt.format(&Rfc3339).map(|s| s.into()).map_err(|e| {
            MemoryError::runtime_error_with_source("Invalid datetime".to_string(), e)
        })?,
        MemoryValue::LocalDateTime(dt) => dt.to_string().into(),
        MemoryValue::Duration(d) => format!("{}", d.whole_nanoseconds()).into(),
    })
}

/// Convert a [`neo4rs::BoltType`] directly into a [`MemoryValue`].
///
/// This is the inverse of [`memory_value_to_bolt`].
pub(crate) fn bolt_to_memory_value(
    bolt: BoltType,
) -> Result<MemoryValue, MemoryError<neo4rs::Error>> {
    Ok(match bolt {
        BoltType::String(s) => MemoryValue::String(s.value),
        BoltType::Integer(i) => MemoryValue::Integer(i.value),
        BoltType::Float(f) => MemoryValue::Float(f.value),
        BoltType::Boolean(b) => MemoryValue::Boolean(b.value),
        BoltType::Bytes(b) => MemoryValue::Bytes(b.value.to_vec()),
        BoltType::List(list) => MemoryValue::List(
            list.value
                .into_iter()
                .map(bolt_to_memory_value)
                .collect::<Result<Vec<MemoryValue>, _>>()?,
        ),
        BoltType::Map(map) => {
            return Err(MemoryError::runtime_error(format!(
                "Unsupported bolt type: Map({:?})",
                map
            )));
        }
        other => {
            return Err(MemoryError::runtime_error(format!(
                "Unsupported bolt type: {:?}",
                other
            )));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

    #[test]
    fn round_trip_string() {
        let v = MemoryValue::String("hello".to_string());
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_list() {
        let v = MemoryValue::List(vec![MemoryValue::Integer(1), MemoryValue::Boolean(true)]);
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_boolean() {
        let v = MemoryValue::Boolean(true);
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_integer() {
        let v = MemoryValue::Integer(-42);
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_float() {
        let v = MemoryValue::Float(3.14);
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_bytes() {
        let v = MemoryValue::Bytes(vec![1, 2, 3]);
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn convert_date() {
        let d = Date::from_calendar_date(2024, Month::January, 1).unwrap();
        let v = MemoryValue::Date(d);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::String(s) => assert_eq!(s.value, "2024-01-01"),
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn convert_time() {
        let t = Time::from_hms(12, 34, 56).unwrap();
        let v = MemoryValue::Time(t);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::String(s) => assert_eq!(s.value, "12:34:56.0"),
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn convert_offset_time() {
        let t = Time::from_hms(1, 2, 3).unwrap();
        let offset = UtcOffset::from_hms(1, 0, 0).unwrap();
        let v = MemoryValue::OffsetTime { time: t, offset };
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::Map(map) => {
                match map.value.get("time") {
                    Some(BoltType::String(s)) => assert_eq!(s.value, "1:02:03.0"),
                    other => panic!("bad time entry: {other:?}"),
                }
                match map.value.get("offset") {
                    Some(BoltType::String(s)) => assert_eq!(s.value, "+01:00:00"),
                    other => panic!("bad offset entry: {other:?}"),
                }
            }
            other => panic!("expected map, got {other:?}"),
        }
    }

    #[test]
    fn convert_datetime() {
        let dt = OffsetDateTime::UNIX_EPOCH;
        let v = MemoryValue::DateTime(dt);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::String(s) => assert_eq!(s.value, "1970-01-01T00:00:00Z"),
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn convert_local_datetime() {
        let date = Date::from_calendar_date(2024, Month::May, 5).unwrap();
        let time = Time::from_hms(6, 7, 8).unwrap();
        let dt = PrimitiveDateTime::new(date, time);
        let v = MemoryValue::LocalDateTime(dt);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::String(s) => assert_eq!(s.value, "2024-05-05 6:07:08.0"),
            other => panic!("expected string, got {other:?}"),
        }
    }

    #[test]
    fn convert_duration() {
        let d = Duration::seconds(5);
        let v = MemoryValue::Duration(d);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::String(s) => assert_eq!(s.value, d.whole_nanoseconds().to_string()),
            other => panic!("expected string, got {other:?}"),
        }
    }
}
