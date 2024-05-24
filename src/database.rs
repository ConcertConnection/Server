use std::collections::HashMap;

use anyhow::{Context, Result};
use scylla::{QueryResult, SessionBuilder};
use scylla::prepared_statement::PreparedStatement;
use scylla::serialize::row::SerializeRow;
use scylla::transport::session::Session;
use struct_iterable::Iterable;

pub use row_structs::*;

use crate::config::{CommonConfig, database::TableConfig};
use crate::config::database::Column;
use crate::domain::TableError;

mod row_structs;

pub struct DatabaseConnection {
    //* holds the db session and prepared statements
    //* other mods use this to access the database
    session: Session,
    keyspace: String,
    pub statements: PreparedStatements
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
    pub async fn execute<T: SerializeRow + Nameable, C: SelectQueryChange>(
        &mut self,
        table_name: &String,
        kind: &mut String,
        row_values: Option<&T>,
        conditions: Option<&C>
    ) -> Result<QueryResult, TableError> {
        // This function executes any of the prepared statements.
        // If insert is the type a row value must be included, function panics if not.
        // If select is the type a result of a vec of results.
        // Does not support batch insert.
        self.ensure_keyspace().await?;
        if kind == "insert" && row_values.is_none() {
            return Err(TableError::MissingRowValues)
        }

        let adjusted_kind = if conditions.is_some() && kind.eq_ignore_ascii_case("select") {
            format!("{}_where", kind)
        } else {
            kind.to_string()
        };

        let prepared_statement = {
            let table = self
                .statements
                .table_queries
                .get_mut(table_name)
                .ok_or(TableError::TableNameNotFound(table_name.to_string()))?;

            if adjusted_kind != "select_where" {
                table
                    .get_query(&adjusted_kind)
                    .map_err(|e| TableError::QueryPrepError(
                        "select where".to_string(), e.to_string()
                    ))?
            } else {
                table
                    .get_query_where(&conditions.unwrap().get_enum(), &self.session)
                    .await?
            }
        };
        println!("{:?}", prepared_statement);
        println!("{:#?}", conditions);

        let execute_result = match (row_values, conditions) {
            (Some(row), _) if !Self::table_name_contains_row_value_name(table_name, row) => {
                Err(TableError::RowValueMismatch(table_name.to_string(), row.get_name()))
            }
            (Some(row), _) => self
                .session
                .execute(prepared_statement, row)
                .await
                .map_err(|err| TableError::QueryExecutionError(err.to_string())),
            (None, Some(cond)) => self
                .session
                .execute(prepared_statement, cond)
                .await
                .map_err(|err| TableError::QueryExecutionError(err.to_string())),
            (None, None) => self
                .session
                .execute(prepared_statement, &[])
                .await
                .map_err(|err| TableError::QueryExecutionError(err.to_string())),
        };

        execute_result
    }

    async fn ensure_keyspace(&self) -> Result<(), TableError> {
        if *self.session.get_keyspace().unwrap() == self.keyspace {
            return Ok(())
        }
        Ok(self.session.use_keyspace(&self.keyspace, false)
            .await.map_err(|err| TableError::QueryExecutionError(err.to_string()))?)

    }

    fn table_name_contains_row_value_name(
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
    pub async fn build(session: &Session, config: &CommonConfig) -> Result<Self, TableError> {
        let mut  table_queries: HashMap<String, TableQueries> = HashMap::new();
        for (name, value) in config.database.tables.iter() {
            if let Some(value) = value.downcast_ref::<TableConfig>() {
                let queries = TableQueries::build(session, value)
                    .await
                    .expect(&format!("Could not build table queries, {}", name));
                table_queries.insert(name.into(), queries);
            } else {
                return Err(TableError::TableDowncastError("Could not downcast to TableConfig.".to_string()));
            }

        }
        Ok(PreparedStatements{ table_queries })
    }

}

pub struct TableQueries {
    insert: PreparedStatement,
    select: PreparedStatement,
    select_where: HashMap<SelectQueries, PreparedStatement>,
    create: PreparedStatement,
    column_names: String,
    table_name: String
}

impl TableQueries {
    pub async fn build(session: &Session, table_config: &TableConfig) -> Result<Self, TableError> {
        let column_names = Self::get_column_names(&table_config.columns);
        let create_statement = Self::make_create_query(table_config);
        let create = session.prepare(create_statement)
            .await
            .map_err(|_| TableError::QueryPrepError("create".to_string(), table_config.name.to_string()))?;
        let response = session.execute(&create, &[])
            .await
            .expect("Did not create table");

        let insert_statement = Self::make_insert_query(table_config, &column_names);
        let insert = session.prepare(insert_statement)
            .await
            .map_err(|_| TableError::QueryPrepError("Insert".to_string(), table_config.name.to_string()))?;

        let select_statement = Self::make_select_query(table_config, &column_names);
        let select = session.prepare(select_statement)
            .await
            .map_err(|_| TableError::QueryPrepError("select".to_string(), table_config.name.to_string()))?;


        let select_where = HashMap::new();

        Ok(TableQueries {
            insert,
            select,
            create,
            select_where,
            column_names,
            table_name: table_config.name.to_string()
        }
        )
    }
    pub fn get_column_names(columns: &Vec<Column>) -> String {
        let col_names: String = columns.iter()
            .map(|column| column.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
        col_names
    }

    pub fn where_select_clause(primary_key: &String) -> String {
        primary_key.split(", ")
            .map(|s| format!("{s} = ?"))
            .collect::<Vec<String>>()
            .join("AND ")
    }
    fn make_insert_query(table_config: &TableConfig, column_names: &String) -> String {
        let question_marks = vec!["?"; table_config.columns.len()]
            .join(", ");
        format!("INSERT INTO {} ({}) VALUES ({})", &table_config.name, column_names, question_marks)
    }

    fn make_select_query(table_config: &TableConfig, column_names: &String) -> String {
        format!("SELECT {} FROM {}", column_names, &table_config.name)
    }

    fn make_conditional_select_query(table_config: &TableConfig, column_names: &String) -> String {
        let select = Self::where_select_clause(&table_config.primary_key);
        format!("SELECT {} FROM {} WHERE {}", column_names, &table_config.name, select)
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

    pub async fn get_query_where(&mut self, select_queries: &SelectQueries, session: &Session) -> Result<&PreparedStatement, TableError> {
        if self.select_where.get(select_queries).is_some() {
            Ok(&self.select_where[select_queries])
        } else {
            let cloned_select_queries = select_queries.clone();
            let statement = self.perpare_where_statement(select_queries, session)
                .await?;
            self.select_where.insert(cloned_select_queries, statement);
            Ok(&self.select_where[select_queries])
        }
    }

    async fn perpare_where_statement(
        &self,
        select_queries: &SelectQueries,
        session: &Session
    ) -> Result<PreparedStatement, TableError> {
        let where_clause = match select_queries {
            SelectQueries::Concert(data) => Self::create_where(data),
            SelectQueries::User(data) => Self::create_where(data),
            SelectQueries::ClaimedPass(data) => Self::create_where(data)
        };
        let statement = format!("SELECT {} FROM {} WHERE {}", &self.column_names, &self.table_name, where_clause);
        session.prepare(statement).await.map_err(|err| TableError::QueryPrepError("select".to_string(), self.table_name.clone()))
    }

    fn create_where<T: Iterable>(select_where: &T) -> String {
        select_where.iter()
            .filter_map(|(name, value)| {
                if let Some(&boolean_value) = value.downcast_ref::<bool>() {
                    if boolean_value {
                        let mut new_name = name.to_string();
                        new_name.push_str(" = ?");
                        return Some(new_name);
                    }
                }
                None
            })
            .collect::<Vec<String>>()
            .join(" AND ")
    }
}


#[cfg(test)]
mod tests{
    use chrono::{TimeZone, Utc};
    use claims::{assert_none, assert_some};
    use fake::Fake;
    use fake::faker::boolean::en::Boolean;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::raw::{FirstName, LastName};
    use fake::locales::EN;
    use scylla::frame::value::CqlTimestamp;
    use struct_iterable::Iterable;
    use uuid::Uuid;

    use crate::config;
    use crate::config::CommonConfig;
    use crate::database::{ClaimedPassConditions, Concert, ConcertConditions, DatabaseConnection, UserConditions};
    use crate::database::{ClaimedPass, User};

    fn first_name() -> String { FirstName(EN).fake() }
    fn last_name() -> String { LastName(EN).fake() }

    fn email() -> String { SafeEmail().fake()}

    fn boolean() -> bool { Boolean(5).fake()}

    async fn create_new_db_connection(config: &mut CommonConfig) -> DatabaseConnection {
        config.database.keyspace = first_name();
        DatabaseConnection::build(&config)
            .await
            .expect("Could not create database connection")
    }

    #[tokio::test]
    async fn prepared_statements_has_all_of_the_tables() {
        let mut config = config::load_configuration()
            .expect("Failed to read config");

        let mut database_connection = create_new_db_connection(&mut config)
            .await;


        for (name, value) in config.database.tables.iter() {
            assert!(database_connection.statements.table_queries.contains_key(name));
        }
    }

    #[tokio::test]
    async fn insert_statement_works() {
        let mut config = config::load_configuration()
            .expect("Failed to read config");
        let mut database_connection = create_new_db_connection(&mut config)
            .await;

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

        let claimed_pass = ClaimedPass::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            user_uuid,
            Uuid::new_v4(),
            Utc.with_ymd_and_hms(2024, 6, 10 , 20, 30, 0).unwrap(),
            "RedRocks".into(),
            "Sting".into(),
            false
        );
        let user_table = String::from("user_table");
        let mut insert = String::from("insert");
        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            &mut insert,
            Some(&user),
            None
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);
        let mut select = String::from("select");
        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            &mut select,
            None,
            None
        )
            .await
            .expect("Could not execute select query");

        assert_some!(&query_result.rows);

        assert_eq!(user, query_result.rows_typed::<User>().unwrap().next().unwrap().unwrap());
        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            &mut insert,
            Some(&user),
            None
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);

        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            &mut select,
            None,
            None
        )
            .await
            .expect("Could not execute select query");

        assert_some!(&query_result.rows);

        assert_eq!(user, query_result.rows_typed::<User>().unwrap().next().unwrap().unwrap());

        let table_query = &database_connection.statements.table_queries;
        let insert = &mut String::from("insert");
        let claimed_passes_s = String::from("claimed_passes");
        let query_result = database_connection.execute::<ClaimedPass, ClaimedPassConditions>(
            &claimed_passes_s,
            insert,
            Some(&claimed_pass),
            None
        )
            .await
            .expect("Could not execute insert query");
        println!("{:?}",database_connection.session.get_keyspace());

        assert_none!(&query_result.rows);
        let query_result = database_connection.execute::<ClaimedPass, ClaimedPassConditions>(
            &claimed_passes_s,
            &mut select,
            None,
            None
        )
            .await
            .expect("Could not execute select query");
        println!("{:?}", &query_result);
        assert_some!(&query_result.rows);
        assert_eq!(claimed_pass, query_result.rows_typed::<ClaimedPass>().unwrap().next().unwrap().unwrap())

    }

    #[tokio::test]
    async fn test_conditional_where() {
        let mut config = config::load_configuration()
            .expect("Failed to read config");
        let mut database_connection = create_new_db_connection(&mut config)
            .await;

        let user_uuid1 = Uuid::new_v4();

        let user1 = User::new(
            user_uuid1,
            first_name(),
            last_name(),
            email(),
            boolean(),
            boolean(),
            Some(Utc::now())
        );

        let user_uuid2 = Uuid::new_v4();

        let user2 = User::new(
            user_uuid2,
            first_name(),
            last_name(),
            email(),
            boolean(),
            boolean(),
            Some(Utc::now())
        );

        let insert = &mut String::from("insert");
        let user_table = String::from("user_table");
        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            insert,
            Some(&user1),
            None
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);
        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            insert,
            Some(&user2),
            None
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);

        let conditions = UserConditions::new(user_uuid1);
        let select_where = &mut String::from("select");
        let query_result = database_connection.execute::<User, UserConditions>(
            &user_table,
            select_where,
            None,
            Some(&conditions)
        )
            .await
            .expect("Could not run select query");
        assert_eq!(query_result.rows.as_ref().unwrap().len(), 1);
        assert_eq!(user1, query_result.rows_typed::<User>().unwrap().next().unwrap().unwrap())

    }
    #[tokio::test]
    async fn test_conditional_where_concert() {
        let mut config = config::load_configuration()
            .expect("Failed to read config");
        let mut database_connection = create_new_db_connection(&mut config)
            .await;

        let concert_uuid1 = Uuid::new_v4();
        let venue_uuid = Uuid::new_v4();

        let concert1 = Concert::new(
            concert_uuid1,
            venue_uuid,
            CqlTimestamp(10000),
            first_name(),
            "None".to_string(),
            "Some_bio".to_string(),
            "None".to_string(),
            33,
            33
        );

        let concert_uuid2 = Uuid::new_v4();

        let concert2 = Concert::new(
            concert_uuid2,
            venue_uuid,
            CqlTimestamp(190888),
            first_name(),
            "None".to_string(),
            "Some_bio".to_string(),
            "None".to_string(),
            33,
            33
        );
        let insert = &mut String::from("insert");
        let concert_table = String::from("concert_table");
        let query_result = database_connection.execute::<Concert, ConcertConditions>(
            &concert_table,
            insert,
            Some(&concert1),
            None
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);
        let query_result = database_connection.execute::<Concert, ConcertConditions>(
            &concert_table,
            insert,
            Some(&concert2),
            None
        )
            .await
            .expect("Could not execute insert query");
        assert_none!(&query_result.rows);

        let conditions = ConcertConditions::new(Some(concert_uuid1), None);
        let select_where = &mut String::from("select");
        let query_result = database_connection.execute::<Concert, ConcertConditions>(
            &concert_table,
            select_where,
            None,
            Some(&conditions)
        )
            .await
            .expect("Could not run select query");
        assert_eq!(query_result.rows.as_ref().unwrap().len(), 1);
        assert_eq!(concert1, query_result.rows_typed::<Concert>().unwrap().next().unwrap().unwrap())

    }
}