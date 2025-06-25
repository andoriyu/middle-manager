pub fn assert_no_defs<T: mm_utils::IntoJsonSchema>() {
    let schema = T::json_schema();
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
