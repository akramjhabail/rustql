use std::fmt;

#[derive(Debug, Clone)]
pub enum RustQLError {
    // Parser errors — 400
    ParseError(String),
    UnexpectedToken(String),
    UnexpectedEOF,

    // Schema errors — 404
    SchemaError(String),
    TypeNotFound(String),
    FieldNotFound(String),

    // Execution errors — 500
    ExecutionError(String),
    ResolverError(String),

    // Validation errors — 400
    ValidationError(String),
    RequiredFieldMissing(String),

    // Auth errors — 401
    Unauthorized(String),
    InvalidToken(String),

    // Not found — 404
    NotFound(String),

    // Rate limit — 429
    RateLimitExceeded,
}

impl RustQLError {
    pub fn status_code(&self) -> u16 {
        match self {
            // 400 Bad Request
            RustQLError::ParseError(_) => 400,
            RustQLError::UnexpectedToken(_) => 400,
            RustQLError::UnexpectedEOF => 400,
            RustQLError::ValidationError(_) => 400,
            RustQLError::RequiredFieldMissing(_) => 400,

            // 401 Unauthorized
            RustQLError::Unauthorized(_) => 401,
            RustQLError::InvalidToken(_) => 401,

            // 404 Not Found
            RustQLError::TypeNotFound(_) => 404,
            RustQLError::FieldNotFound(_) => 404,
            RustQLError::NotFound(_) => 404,
            RustQLError::SchemaError(_) => 404,

            // 429 Too Many Requests
            RustQLError::RateLimitExceeded => 429,

            // 500 Internal Server Error
            RustQLError::ExecutionError(_) => 500,
            RustQLError::ResolverError(_) => 500,
        }
    }

    pub fn error_type(&self) -> &str {
        match self {
            RustQLError::ParseError(_) => "PARSE_ERROR",
            RustQLError::UnexpectedToken(_) => "UNEXPECTED_TOKEN",
            RustQLError::UnexpectedEOF => "UNEXPECTED_EOF",
            RustQLError::SchemaError(_) => "SCHEMA_ERROR",
            RustQLError::TypeNotFound(_) => "TYPE_NOT_FOUND",
            RustQLError::FieldNotFound(_) => "FIELD_NOT_FOUND",
            RustQLError::ExecutionError(_) => "EXECUTION_ERROR",
            RustQLError::ResolverError(_) => "RESOLVER_ERROR",
            RustQLError::ValidationError(_) => "VALIDATION_ERROR",
            RustQLError::RequiredFieldMissing(_) => "REQUIRED_FIELD_MISSING",
            RustQLError::Unauthorized(_) => "UNAUTHORIZED",
            RustQLError::InvalidToken(_) => "INVALID_TOKEN",
            RustQLError::NotFound(_) => "NOT_FOUND",
            RustQLError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
        }
    }
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
            RustQLError::Unauthorized(msg) =>
                write!(f, "Unauthorized: {}", msg),
            RustQLError::InvalidToken(msg) =>
                write!(f, "Invalid Token: {}", msg),
            RustQLError::NotFound(msg) =>
                write!(f, "Not Found: {}", msg),
            RustQLError::RateLimitExceeded =>
                write!(f, "Rate Limit Exceeded"),
        }
    }
}

impl std::error::Error for RustQLError {}

pub type RustQLResult<T> = Result<T, RustQLError>;