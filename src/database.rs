use anyhow::Result;
use scylla::macros::FromRow;
use scylla::transport::session::Session;
use scylla::SessionBuilder;
use std::env;
use std::fmt::format;
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::errors::NewSessionError;
use crate::config::Config;

pub struct DatabaseConnection {
    //* holds the db session and prepared statements
    //* other mods use this to access the database
    session: Session,
    statements: PreparedStatements
}
impl DatabaseConnection {
    pub async fn build(config: &Config) -> Result<Self, anyhow::Error>{
        let session: Session = SessionBuilder::new()
            .known_node(config.database.db_addr)
            .build()
            .await?;

        let keyspace = config.database.keyspace;

        session.query(
            format!("CREATE KEYSPACE IF NOT EXISTS {keyspace} WITH REPLICATION = {{'class': 'NetworkTopologyStrategy', 'replication_factor': 1}}"),
            &[]
        ).await?;
        session.query(format!("USE KEYSPACE {keyspace}"), &[]).await?;
        let statements = PreparedStatements::new(&session)?;
        Ok(DatabaseConnection { session, statements })
    }
}
struct PreparedStatements {
    //* Contains the prepared statements used in our queries
    pub users: PreparedStatement,

}

impl PreparedStatements {
    pub async fn build(session: &Session, config: &Config) -> Result<Self, anyhow::Error> {
        let user_table = config.database.tables.user_table.name;
        let user_table_columns: &Vec<String> = config.database.tables.user_table.columns;
        let user_insert = session
            .prepare(
                format!(
                    "INSERT INTO {} {} VALUES ({})",
                    user_table, user_table_columns.join(", "), vec!["?"; user_table_columns.len()].join(", ")
                )
            ).await?;
    }

    Ok(PreparedStatements { user_insert, })
}