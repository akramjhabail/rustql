pub struct Migration {
    pub version: i64,
    pub name: String,
    pub sql: String,
}

impl Migration {
    pub fn new(version: i64, name: &str, sql: &str) -> Self {
        Migration {
            version,
            name: name.to_string(),
            sql: sql.to_string(),
        }
    }
}
