use schemars::{
    JsonSchema,
    generate::SchemaSettings,
    transform::{AddNullable, RecursiveTransform},
};
use serde_json;

/// Trait for converting types to JSON Schema
pub trait IntoJsonSchema {
    /// Convert the type to a JSON Schema object
    fn json_schema() -> serde_json::Map<String, serde_json::Value>;
}

/// Blanket implementation for all types that implement JsonSchema
impl<T: JsonSchema> IntoJsonSchema for T {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let generator = SchemaSettings::draft2020_12()
            .with(|s| {
                s.meta_schema = None;
                s.inline_subschemas = true;
            })
            .with_transform(RecursiveTransform(AddNullable::default()))
            .into_generator();
        serde_json::to_value(generator.into_root_schema_for::<T>())
            .expect("schema serialization")
            .as_object()
            .cloned()
            .expect("schema object")
    }
}
