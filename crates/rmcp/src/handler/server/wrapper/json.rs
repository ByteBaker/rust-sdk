use std::borrow::Cow;

use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    handler::server::tool::IntoCallToolResult,
    model::{CallToolResponse, CallToolResult},
};

/// Json wrapper for structured output
///
/// When used with tools, this wrapper indicates that the value should be
/// serialized as structured JSON content with an associated schema.
/// The framework will place the JSON in the `structured_content` field
/// of the tool result rather than the regular `content` field.
#[expect(clippy::exhaustive_structs, reason = "intentionally exhaustive")]
pub struct Json<T>(pub T);

// Implement JsonSchema for Json<T> to delegate to T's schema
impl<T: JsonSchema> JsonSchema for Json<T> {
    fn schema_name() -> Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(generator)
    }
}

// Implementation for Json<T> to create structured content
impl<T: Serialize + JsonSchema + 'static> IntoCallToolResult for Json<T> {
    fn into_call_tool_result(self) -> Result<CallToolResponse, crate::ErrorData> {
        let value = serde_json::to_value(self.0).map_err(|e| {
            crate::ErrorData::internal_error(
                format!("Failed to serialize structured content: {}", e),
                None,
            )
        })?;

        Ok(CallToolResult::structured(value).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_failure_is_an_internal_error() {
        #[derive(JsonSchema)]
        struct Unserializable;

        impl Serialize for Unserializable {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("intentional failure"))
            }
        }

        let error = Json(Unserializable).into_call_tool_result().unwrap_err();
        assert_eq!(
            error,
            crate::ErrorData::internal_error(
                "Failed to serialize structured content: intentional failure",
                None,
            )
        );
    }
}
