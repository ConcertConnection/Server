



pub struct DatabaseConfig {
    pub keyspace: String,
    pub user_table: TableConfig,
    pub claimed_pass_table: TableConfig,
    pub unclaimed_pass_table: TableConfig,
    pub concert_table: TableConfig,
    pub venue_table: TableConfig,
}

pub struct TableConfig {
    name: String,
    columns: Vec<String>,
    primary_key: String
}