use schemars::JsonSchema;
use serde_json;

/// Trait for converting types to JSON Schema
pub trait IntoJsonSchema {
    /// Convert the type to a JSON Schema object
    fn json_schema() -> serde_json::Map<String, serde_json::Value>;
}

/// Blanket implementation for all types that implement JsonSchema
impl<T: JsonSchema> IntoJsonSchema for T {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        serde_json::to_value(schemars::schema_for!(T))
            .expect("schema serialization")
            .as_object()
            .cloned()
            .expect("schema object")
    }
}
