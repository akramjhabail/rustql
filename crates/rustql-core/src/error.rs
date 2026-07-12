use std::fmt;

#[derive(Debug, Clone)]
pub enum RustQLError {
    // Parser errors
    ParseError(String),
    UnexpectedToken(String),
    UnexpectedEOF,

    // Schema errors
    SchemaError(String),
    TypeNotFound(String),
    FieldNotFound(String),

    // Execution errors
    ExecutionError(String),
    ResolverError(String),

    // Validation errors
    ValidationError(String),
    RequiredFieldMissing(String),
}

impl fmt::Display for RustQLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RustQLError::ParseError(msg) =>
                write!(f, "Parse Error: {}", msg),
            RustQLError::UnexpectedToken(token) =>
                write!(f, "Unexpected Token: {}", token),
            RustQLError::UnexpectedEOF =>
                write!(f, "Unexpected End of Input"),
            RustQLError::SchemaError(msg) =>
                write!(f, "Schema Error: {}", msg),
            RustQLError::TypeNotFound(name) =>
                write!(f, "Type Not Found: {}", name),
            RustQLError::FieldNotFound(name) =>
                write!(f, "Field Not Found: {}", name),
            RustQLError::ExecutionError(msg) =>
                write!(f, "Execution Error: {}", msg),
            RustQLError::ResolverError(msg) =>
                write!(f, "Resolver Error: {}", msg),
            RustQLError::ValidationError(msg) =>
                write!(f, "Validation Error: {}", msg),
            RustQLError::RequiredFieldMissing(name) =>
                write!(f, "Required Field Missing: {}", name),
        }
    }
}

impl std::error::Error for RustQLError {}

pub type RustQLResult<T> = Result<T, RustQLError>;