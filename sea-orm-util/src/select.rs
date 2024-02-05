use std::marker::PhantomData;

use sea_orm::{
    sea_query::{
        Alias, IntoIden, MysqlQueryBuilder, PostgresQueryBuilder, SelectStatement,
        SqliteQueryBuilder,
    },
    ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, ExecResult, QueryResult, Select,
    Statement,
};

pub trait FromSubquery {
    fn from_subquery(self, subquery: SelectStatement) -> Self;

    fn from_subquery_as<T: IntoIden>(self, subquery: SelectStatement, alias: T) -> Self;
}

struct _Select<E>
where
    E: EntityTrait,
{
    query: SelectStatement,
    entity: PhantomData<E>,
}

impl<E> FromSubquery for Select<E>
where
    E: EntityTrait,
{
    fn from_subquery(self, subquery: SelectStatement) -> Self {
        let entity = E::default();
        self.from_subquery_as(subquery, Alias::new(entity.table_name()))
    }

    fn from_subquery_as<T: IntoIden>(self, subquery: SelectStatement, alias: T) -> Self {
        let mut r = unsafe { std::mem::transmute::<_, _Select<E>>(self) };

        r.query.from_clear().from_subquery(subquery, alias);

        unsafe { std::mem::transmute::<_, Select<E>>(r) }
    }
}

#[async_trait::async_trait]
pub trait Execute {
    async fn query_all(&self, conn: &DatabaseConnection) -> Result<Vec<QueryResult>, DbErr>;

    async fn query_one(&self, conn: &DatabaseConnection) -> Result<Option<QueryResult>, DbErr>;

    async fn execute(&self, conn: &DatabaseConnection) -> Result<ExecResult, DbErr>;
}

#[async_trait::async_trait]
impl Execute for SelectStatement {
    async fn query_all(&self, conn: &DatabaseConnection) -> Result<Vec<QueryResult>, DbErr> {
        let db_backend = conn.get_database_backend();
        let (query, values) = match db_backend {
            sea_orm::DatabaseBackend::MySql => self.build(MysqlQueryBuilder),
            sea_orm::DatabaseBackend::Postgres => self.build(PostgresQueryBuilder),
            sea_orm::DatabaseBackend::Sqlite => self.build(SqliteQueryBuilder),
        };

        conn.query_all(Statement::from_sql_and_values(db_backend, query, values))
            .await
    }

    async fn query_one(&self, conn: &DatabaseConnection) -> Result<Option<QueryResult>, DbErr> {
        let db_backend = conn.get_database_backend();
        let (query, values) = match db_backend {
            sea_orm::DatabaseBackend::MySql => self.build(MysqlQueryBuilder),
            sea_orm::DatabaseBackend::Postgres => self.build(PostgresQueryBuilder),
            sea_orm::DatabaseBackend::Sqlite => self.build(SqliteQueryBuilder),
        };

        conn.query_one(Statement::from_sql_and_values(db_backend, query, values))
            .await
    }

    async fn execute(&self, conn: &DatabaseConnection) -> Result<ExecResult, DbErr> {
        let db_backend = conn.get_database_backend();
        let (query, values) = match db_backend {
            sea_orm::DatabaseBackend::MySql => self.build(MysqlQueryBuilder),
            sea_orm::DatabaseBackend::Postgres => self.build(PostgresQueryBuilder),
            sea_orm::DatabaseBackend::Sqlite => self.build(SqliteQueryBuilder),
        };

        conn.execute(Statement::from_sql_and_values(db_backend, query, values))
            .await
    }
}
