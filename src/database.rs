use std::collections::HashMap;
use anyhow::{Result, anyhow};
use scylla::serialize::row::SerializeRow;
use scylla::transport::session::Session;
use scylla::{QueryResult, SessionBuilder};
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::errors::QueryError;
use crate::config::{ CommonConfig, database::TableConfig};
use crate::config::database::Column;

use struct_iterable::Iterable;

mod row_structs;
use row_structs::Nameable;

pub trait ConvertToRowInsert {
    fn convert_to_db_row(&self) -> String;
}

pub struct DatabaseConnection {
    //* holds the db session and prepared statements
    //* other mods use this to access the database
    session: Session,
    keyspace: String,
    statements: PreparedStatements
}
impl DatabaseConnection {
    pub async fn build(config: &CommonConfig) -> Result<Self>{
        let session: Session = SessionBuilder::new()
            .known_node(config.database.database_addr())
            .build()
            .await?;

        let keyspace = String::from(config.database.keyspace());

        session.query(
            format!("CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{'class': 'NetworkTopologyStrategy', 'replication_factor': 1}}", keyspace),
            &[]
        ).await?;
        session.use_keyspace(&keyspace, false).await?;
        let statements = PreparedStatements::build(&session, config)
            .await?;
        Ok(DatabaseConnection { session, keyspace, statements })
    }
    pub async fn execute<T: SerializeRow + Nameable>(
        &self,
        table_name: &String,
        kind: &String,
        row_values :Option<&T>
    ) -> Result<QueryResult, QueryError> {
        // This function executes any of the prepared statements.
        // If insert is the type a row value must be included, function panics if not.
        // If select is the type a result of a vec of results.
        // Does not support batch insert.
        if kind == "insert" && row_values.is_none() {
            panic!("Row values required for insert")
        }
        let table = &self.statements.table_queries[table_name];
        let prepared_statement = match table.get_query(kind) {
            Ok(statement) => statement,
            Err(e) => panic!("{e}")
        };
        if *self.session.get_keyspace().unwrap() != self.keyspace {
            self.session.use_keyspace(&self.keyspace, false)
                .await?
        }
        if let Some(row) = row_values {
            if !self.table_name_contains_row_value_name(table_name, row) {
                panic!("The passed Serialize row struct does not match the table name passed")
            }
            self.session.execute(prepared_statement, row)
                .await
        } else {
            self.session.execute(prepared_statement, &[])
                .await
        }
    }

    fn table_name_contains_row_value_name(
        &self,
        table_name: &String,
        row_values: &impl Nameable) -> bool{
        table_name.contains(&row_values.get_name())
    }
}
pub struct PreparedStatements {
    //* Contains the prepared statements used in our queries
    pub table_queries: HashMap<String, TableQueries>,

}


impl PreparedStatements {
    pub async fn build(session: &Session, config: &CommonConfig) -> Result<Self> {
        let mut  table_queries: HashMap<String, TableQueries> = HashMap::new();
        for (name, value) in config.database.tables.iter() {
            if let Some(value) = value.downcast_ref::<TableConfig>() {
                let queries = TableQueries::build(session, value)
                    .await
                    .expect(&format!("Could not build table queries, {}", name));
                table_queries.insert(name.into(), queries);
            } else {
                return Err(anyhow!("Could not downcast to TableConfig."));
            }

        }
        Ok(PreparedStatements{ table_queries })
    }

}

pub struct TableQueries {
    insert: PreparedStatement,
    select: PreparedStatement,
    create: PreparedStatement
}

impl TableQueries {
    pub async fn build(session: &Session, table_config: &TableConfig) -> Result<Self> {
        let create_statement = Self::make_create_query(table_config);
        let create = session.prepare(create_statement).await?;
        let response = session.execute(&create, &[])
            .await
            .expect("Did not create table");

        let insert_statement = Self::make_insert_query(table_config);
        let insert = session.prepare(insert_statement).await?;

        let select_statement = Self::make_select_query(table_config);
        let select = session.prepare(select_statement).await?;


        Ok(TableQueries { insert, select, create })
    }
    pub fn get_column_names(columns: &Vec<Column>) -> String {
        let col_names: String = columns.iter()
            .map(|column| column.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
        col_names
    }
    fn make_insert_query(table_config: &TableConfig) -> String {
        let column_names: String = Self::get_column_names(&table_config.columns);
        let question_marks = vec!["?"; table_config.columns.len()]
            .join(", ");
        format!("INSERT INTO {} ({}) VALUES ({})", &table_config.name, column_names, question_marks)
    }

    fn make_select_query(table_config: &TableConfig) -> String {
        let column_names = Self::get_column_names(&table_config.columns);
        format!("SELECT {} FROM {}", column_names, &table_config.name)
    }

    fn make_create_query(table_config: &TableConfig) -> String {
        let column_name_type: String = table_config.columns.iter()
            .map(|col| format!("{} {}", col.name, col.dtype))
            .collect::<Vec<String>>()
            .join(", ");
        format!("CREATE TABLE IF NOT EXISTS {} ({}, primary key ({}))", &table_config.name, column_name_type, &table_config.primary_key)
    }

    pub fn get_query(&self, query_name: &String) -> Result<&PreparedStatement, String> {
        let query = match query_name.to_lowercase().as_str() {
            "insert" => &self.insert,
            "select" => &self.select,
            "create" => &self.create,
            other => return Err(format!("{} is not a supported query", other))
        };

        Ok(query)
    }
}


#[cfg(test)]
mod tests{
    use chrono::Utc;
    use struct_iterable::Iterable;
    use crate::config;
    use crate::database::DatabaseConnection;
    use crate::database::row_structs::User;
    use uuid::Uuid;
    use fake::faker::name::raw::{ FirstName, LastName };
    use fake::faker::boolean::en::Boolean;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::locales::EN;
    use claims::{assert_err, assert_ok, assert_none, assert_some};

    fn first_name() -> String { FirstName(EN).fake() }
    fn last_name() -> String { LastName(EN).fake() }

    fn email() -> String { SafeEmail().fake()}

    fn boolean() -> bool { Boolean(5).fake()}

    #[tokio::test]
    async fn prepared_statements_has_all_of_the_tables() {
        let config = config::load_configuration()
            .expect("Failed to read config");

        let database_connection = DatabaseConnection::build(&config)
            .await
            .expect("Could not create database connection");


        for (name, value) in config.database.tables.iter() {
            assert!(database_connection.statements.table_queries.contains_key(name));
        }
    }

    #[tokio::test]
    async fn insert_statement_works() {
        let config = config::load_configuration()
            .expect("Failed to read config");
        let database_connection = DatabaseConnection::build(&config)
            .await
            .expect("Could not create database connection");

        let user_uuid = Uuid::new_v4();

        let user = User::new(
            user_uuid,
            first_name(),
            last_name(),
            email(),
            boolean(),
            boolean(),
            Some(Utc::now())
        );

        let query_result = database_connection.execute(
            &String::from("user_table"),
            &String::from("insert"),
            Some(&user)
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);

        let query_result = database_connection.execute::<User>(
            &String::from("user_table"),
            &String::from("select"),
            None
        )
            .await
            .expect("Could not execute select query");

        assert_some!(&query_result.rows);

        assert_eq!(user, query_result.rows_typed::<User>().unwrap().next().unwrap().unwrap())

    }
}