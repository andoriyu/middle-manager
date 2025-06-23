use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use mm_memory::{MemoryError, MemoryValue};
use neo4rs::BoltType;
use std::collections::HashMap;

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
            // Convert Vec<String> to Vec<BoltType>
            let bolt_items: Vec<BoltType> = items.iter().map(|s| s.clone().into()).collect();
            bolt_items.into()
        }
        MemoryValue::Map(map) => {
            // Convert HashMap<String, String> to BoltType::Map
            let mut bolt_map = HashMap::new();
            for (k, v) in map {
                bolt_map.insert(k.clone(), BoltType::String(v.clone().into()));
            }
            bolt_map.into()
        }
        MemoryValue::Date(d) => (*d).into(),
        MemoryValue::Time(t) => (*t).into(),
        MemoryValue::OffsetTime { time, offset } => (*time, *offset).into(),
        MemoryValue::DateTime(dt) => (*dt).into(),
        MemoryValue::LocalDateTime(dt) => (*dt).into(),
        MemoryValue::Duration(d) => (*d).into(),
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
        BoltType::List(list) => {
            // Convert list of BoltType to Vec<String>
            let string_list = list
                .value
                .into_iter()
                .map(|bolt_item| match bolt_to_memory_value(bolt_item)? {
                    MemoryValue::String(s) => Ok::<String, MemoryError<neo4rs::Error>>(s),
                    other => Ok(other.to_string()),
                })
                .collect::<Result<Vec<String>, _>>()?;
            MemoryValue::List(string_list)
        }
        BoltType::Map(map) => {
            // Convert BoltType::Map to HashMap<String, String>
            let mut string_map = HashMap::new();
            for (k, v) in map.value {
                let value = match bolt_to_memory_value(v)? {
                    MemoryValue::String(s) => s,
                    other => other.to_string(),
                };
                string_map.insert(k.to_string(), value);
            }
            MemoryValue::Map(string_map)
        }
        BoltType::Duration(d) => MemoryValue::Duration(d.into()),
        BoltType::Date(d) => {
            let date: NaiveDate = d.try_into().map_err(|e| {
                MemoryError::runtime_error_with_source("Invalid date".to_string(), e)
            })?;
            MemoryValue::Date(date)
        }
        BoltType::Time(t) => {
            let (time, offset): (NaiveTime, FixedOffset) = (&t).into();
            if offset.local_minus_utc() == 0 {
                MemoryValue::Time(time)
            } else {
                MemoryValue::OffsetTime { time, offset }
            }
        }
        BoltType::LocalTime(t) => MemoryValue::Time(t.into()),
        BoltType::DateTime(dt) => {
            let dt: DateTime<FixedOffset> = dt.try_into().map_err(|e| {
                MemoryError::runtime_error_with_source("Invalid datetime".to_string(), e)
            })?;
            MemoryValue::DateTime(dt)
        }
        BoltType::LocalDateTime(dt) => {
            let ndt: NaiveDateTime = dt.try_into().map_err(|e| {
                MemoryError::runtime_error_with_source("Invalid local datetime".to_string(), e)
            })?;
            MemoryValue::LocalDateTime(ndt)
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
    use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
    use std::time::Duration;

    #[test]
    fn round_trip_string() {
        let v = MemoryValue::String("hello".to_string());
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_list() {
        let v = MemoryValue::List(vec!["1".to_string(), "true".to_string()]);
        let bolt = memory_value_to_bolt(&v).unwrap();
        let back = bolt_to_memory_value(bolt).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn round_trip_map() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());
        let v = MemoryValue::Map(map);
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
        let v = MemoryValue::Float(6.9);
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
        let d = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let v = MemoryValue::Date(d);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::Date(date) => {
                let got: NaiveDate = date
                    .try_into()
                    .expect("Failed to convert BoltDate to NaiveDate");
                assert_eq!(got, d);
            }
            other => panic!("expected date, got {other:?}"),
        }
    }

    #[test]
    fn convert_time() {
        let t = NaiveTime::from_hms_opt(12, 34, 56).unwrap();
        let v = MemoryValue::Time(t);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::LocalTime(time) => {
                let got: NaiveTime = time.into();
                assert_eq!(got, t);
            }
            other => panic!("expected local time, got {other:?}"),
        }
    }

    #[test]
    fn convert_offset_time() {
        let t = NaiveTime::from_hms_opt(1, 2, 3).unwrap();
        let offset = FixedOffset::east_opt(3600).unwrap();
        let v = MemoryValue::OffsetTime { time: t, offset };
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::Time(time) => {
                let (got_time, got_offset): (NaiveTime, FixedOffset) = (&time).into();
                assert_eq!(got_time, t);
                assert_eq!(got_offset, offset);
            }
            other => panic!("expected time, got {other:?}"),
        }
    }

    #[test]
    fn convert_datetime() {
        let dt = DateTime::from_naive_utc_and_offset(
            DateTime::UNIX_EPOCH.naive_utc(),
            FixedOffset::east_opt(0).unwrap(),
        );
        let v = MemoryValue::DateTime(dt);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::DateTime(d) => {
                let got: DateTime<FixedOffset> = d
                    .try_into()
                    .expect("Failed to convert BoltDateTime to DateTime<FixedOffset>");
                assert_eq!(got, dt);
            }
            other => panic!("expected datetime, got {other:?}"),
        }
    }

    #[test]
    fn convert_local_datetime() {
        let date = NaiveDate::from_ymd_opt(2024, 5, 5).unwrap();
        let time = NaiveTime::from_hms_opt(6, 7, 8).unwrap();
        let dt = NaiveDateTime::new(date, time);
        let v = MemoryValue::LocalDateTime(dt);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::LocalDateTime(ldt) => {
                let got: NaiveDateTime = ldt
                    .try_into()
                    .expect("Failed to convert BoltLocalDateTime to NaiveDateTime");
                assert_eq!(got, dt);
            }
            other => panic!("expected local datetime, got {other:?}"),
        }
    }

    #[test]
    fn convert_duration() {
        let d = Duration::from_secs(5);
        let v = MemoryValue::Duration(d);
        let bolt = memory_value_to_bolt(&v).unwrap();
        match bolt {
            BoltType::Duration(bd) => {
                let got: Duration = bd.into();
                assert_eq!(got, d);
            }
            other => panic!("expected duration, got {other:?}"),
        }
    }
}
