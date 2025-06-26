use mm_utils::json_schema::schema_generator;
use schemars::JsonSchema;
use serde_json;

pub fn assert_no_defs<T: JsonSchema>() {
    let generator = schema_generator();
    let root_schema = generator.into_root_schema_for::<T>();
    let schema: serde_json::Map<String, serde_json::Value> = serde_json::to_value(root_schema)
        .expect("schema serialization")
        .as_object()
        .cloned()
        .expect("schema object");
    assert!(
        !schema.contains_key("$defs"),
        "Schema should not contain $defs section",
    );
    let schema_str = serde_json::to_string(&schema).expect("Failed to convert schema to string");
    assert!(
        !schema_str.contains("\"$ref\":\"#/$defs/"),
        "Schema should not contain references to $defs",
    );
}
