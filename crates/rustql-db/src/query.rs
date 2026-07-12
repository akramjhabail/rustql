pub struct QueryBuilder {
    table: String,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        QueryBuilder {
            table: table.to_string(),
        }
    }

    pub fn table(&self) -> &str {
        &self.table
    }
}
