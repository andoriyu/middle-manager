#[macro_export]
macro_rules! validate_name {
    ($name:expr) => {
        if $name.is_empty() {
            return Err($crate::error::CoreError::Validation(
                mm_memory::ValidationError(vec![mm_memory::ValidationErrorKind::EmptyEntityName]),
            ));
        }
    };
}
