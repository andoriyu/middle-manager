use schemars::{
    JsonSchema,
    r#gen::SchemaGenerator,
    generate::SchemaSettings,
    transform::{AddNullable, RecursiveTransform},
};
use serde_json;

/// Trait for converting types to JSON Schema
pub trait IntoJsonSchema {
    /// Convert the type to a JSON Schema object
    fn json_schema() -> serde_json::Map<String, serde_json::Value>;
}

/// Return a schema generator configured for Middle Manager schemas.
pub fn schema_generator() -> SchemaGenerator {
    SchemaSettings::draft2020_12()
        .with(|s| {
            s.meta_schema = None;
            s.inline_subschemas = true;
        })
        .with_transform(RecursiveTransform(AddNullable::default()))
        .into_generator()
}

/// Blanket implementation for all types that implement JsonSchema
impl<T: JsonSchema> IntoJsonSchema for T {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let generator = schema_generator();

        // Generate the full root schema which includes all definitions
        let root_schema = generator.into_root_schema_for::<T>();

        // Convert to a Value and extract as an object
        serde_json::to_value(root_schema)
            .expect("schema serialization")
            .as_object()
            .cloned()
            .expect("schema object")
    }
}
