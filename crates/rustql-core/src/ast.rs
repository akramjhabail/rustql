#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub arguments: Vec<(String, Value)>,
    pub selections: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct OperationDefinition {
    pub operation_type: OperationType,
    pub name: Option<String>,
    pub selections: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub operations: Vec<OperationDefinition>,
}

impl Document {
    pub fn new() -> Self {
        Document {
            operations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Int,
    Float,
    Bool,
    ID,
    Custom(String),
    List(Box<FieldType>),
}