use std::collections::HashMap;
use anyhow::Result;
use scylla::macros::FromRow;
use scylla::transport::session::Session;
use scylla::SessionBuilder;
use std::env;
use std::fmt::format;
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::errors::NewSessionError;
use scylla::transport::topology::Table;
use crate::config::{ CommonConfig, database::TableConfig};
use crate::config::database::Column;


mod row_structs;


pub struct DatabaseConnection {
    //* holds the db session and prepared statements
    //* other mods use this to access the database
    session: Session,
    statements: PreparedStatements
}
impl DatabaseConnection {
    pub async fn build(config: &CommonConfig) -> Result<Self, anyhow::Error>{
        let session: Session = SessionBuilder::new()
            .known_node(&config.database.database_addr)
            .build()
            .await?;

        let keyspace = &config.database.keyspace;

        session.query(
            format!("CREATE KEYSPACE IF NOT EXISTS {keyspace} WITH REPLICATION = {{'class': 'NetworkTopologyStrategy', 'replication_factor': 1}}"),
            &[]
        ).await?;
        session.query(format!("USE KEYSPACE {keyspace}"), &[]).await?;
        let statements = PreparedStatements::build(&session, config)
            .await?;
        Ok(DatabaseConnection { session, statements })
    }
    pub async fn execute(session: &Session, prepared_statement: &PreparedStatement, values:)
}
pub struct PreparedStatements {
    //* Contains the prepared statements used in our queries
    pub table_queries: HashMap<String, TableQueries>,

}


impl PreparedStatements {
    pub async fn build(session: &Session, config: &CommonConfig) -> Result<Self> {
        let table_quries: HashMap<String, TableQueries> = HashMap::new();
        for table_config in &config.database.tables {
            let name = &table_config.name;
            table_quries[name]
        }
        Ok(PreparedStatements{})
    }


}

pub struct TableQueries {
    insert: PreparedStatement,
    select: PreparedStatement,
    create: PreparedStatement
}

impl TableQueries {
    pub async fn build(&self, session: &Session, table_config: &TableConfig) -> Result<Self> {
        let insert_statment = self.make_insert_query(table_config);
        let insert = session.prepare(insert_statment).await?;

        let select_statement = self.make_select_query(table_config);
        let select = session.prepare(select_statement).await?;

        let create_statement = self.make_create_query(table_config);
        let create = session.prepare(create_statement).await?;
        Ok(TableQueries { insert, select, create })
    }
    pub fn get_column_names(&self, columns: &Vec<Column>) -> String {
        columns.iter()
            .map(|column| &column.name)
            .collect()
            .join(", ")
    }
    pub fn make_insert_query(&self, table_config: &TableConfig) -> String {
        let column_names: String = self.get_column_names(&table_config.columns);
        let question_marks = vec!["?"; table_config.columns.len()]
            .join(", ");
        format!("INSERT INTO {} ({}) VALUES ({})", &table_config.name, column_names, question_marks)
    }

    pub fn make_select_query(&self, table_config: &TableConfig) -> String {
        let column_names = self.get_column_names(&table_config.columns);
        format!("SELECT {} FROM {}", column_names, &table_config.name)
    }

    pub fn make_create_query(&self, table_config: &TableConfig) -> String {
        let column_name_type = table_config.columns.iter()
            .map(|col| format!("{} {}", col.name, col.dtype))
            .collect()
            .join(", ");
        format!("CREATE TABLE IF NOT EXISTS {} ({}, primary key ({})", &table_config.name, column_name_type, &table_config.primary_key)
    }
}
