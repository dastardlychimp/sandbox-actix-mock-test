use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgArguments, Row, Arguments};
use thiserror;

#[cfg(test)]
use mockall::*;

pub mod handlers;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TR {
    id: i32,
    col1: String,
}

// Associated type must be defined for the mock. We could do anyhow::Error for these?
// Or as seen below, you can mock an implementation of it, which has a different type.
#[cfg_attr(test, automock(type Error=String;))]
#[async_trait]
pub trait Datasource: Send + Sync {
    type Error: std::fmt::Debug;

    async fn select_all_test(&self) -> Result<Vec<TR>, Self::Error>;

    async fn select_last_test(&self) -> Result<TR, Self::Error>;
}

#[derive(Clone)]
pub struct PgDatasource {
    pool: PgPool,
}

impl PgDatasource {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}


// Can't use automock attribute macro since it implements multiple traits
#[cfg(test)]
mock! {
    pub PgDatasource {}

    #[async_trait]
    impl Datasource for PgDatasource {
        type Error = sqlx::Error;

        async fn select_all_test(&self) -> Result<Vec<TR>, sqlx::Error>;
        async fn select_last_test(&self) -> Result<TR, sqlx::Error>;
    }

    #[async_trait]
    impl auth::AuthDatasource for PgDatasource {
        type Error = sqlx::Error;

        async fn key_limit(&self, key: &str) -> Result<Option<auth::KeyLimit>, sqlx::Error>;
    }
}

#[async_trait]
impl Datasource for PgDatasource {
    type Error = sqlx::Error;

    async fn select_all_test(&self) -> Result<Vec<TR>, Self::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as("SELECT * FROM test")
            .fetch_all(&mut conn)
            .await
    }

    async fn select_last_test(&self) -> Result<TR, Self::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as("SELECT * FROM test ORDER BY id DESC")
            .fetch_one(&mut conn)
            .await
    }
}
pub mod auth {
    use super::*;
    pub(crate) static UNLIMITED_KEY: &'static str = "unlimit";

    #[derive(Debug, Clone)]
    pub enum KeyLimit {
        Unlimited,
        Limit(usize),
    }
    #[async_trait]
    pub trait AuthDatasource: Send + Sync {
        type Error: std::fmt::Debug;

        async fn key_limit(&self, key: &str) -> Result<Option<KeyLimit>, Self::Error>;
    }

    #[async_trait]
    impl AuthDatasource for PgDatasource {
        type Error = sqlx::Error;

        async fn key_limit(&self, key: &str) -> Result<Option<KeyLimit>, Self::Error> {
            if key == UNLIMITED_KEY {
                return Ok(Some(KeyLimit::Unlimited));
            }

            let mut conn = self.pool.acquire().await?;

            let mut args = PgArguments::default();
            args.add(key);
            let limit = sqlx::query_with("SELECT limit FROM key_limit WHERE key = $1", args)
                .fetch_optional(&mut conn)
                .await?
                .map(|r| r.try_get("limit").map(|i: i32| i as usize))
                .transpose()?
                .map(KeyLimit::Limit);
            
            Ok(limit)
        }
    }
}

pub(crate) mod model {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    pub enum ModelError<E> {
        #[error("datasource error: {0:?}")]
        DatasourceError(#[from] E),
    }

    pub async fn get_datas_start_with_char<E: std::fmt::Debug>(
        datasource: &dyn Datasource<Error = E>,
        starting_char: char,
    ) -> Result<Vec<String>, ModelError<E>> {
        let rows = datasource.select_all_test().await?;

        let filtered_rows = rows
            .into_iter()
            .filter_map(|row| {
                if row.col1.starts_with(starting_char) {
                    Some(row.col1)
                } else {
                    None
                }
            })
            .collect::<_>();

        Ok(filtered_rows)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_rt;

    #[actix_rt::test]
    async fn test_pg() {
        let pool = PgPool::connect(env!("DATABASE_URL")).await.unwrap();
        let db = PgDatasource::new(pool);
        let results = db.select_all_test().await.unwrap();

        assert!(results.len() > 1)
    }

    #[actix_rt::test]
    async fn test_all_rows_with_char() {
        let pool = PgPool::connect(env!("DATABASE_URL")).await.unwrap();
        let db = PgDatasource::new(pool);
        let results = model::get_datas_start_with_char(&db, 'c').await.unwrap();

        assert_eq!(results, vec!["c", "cantaloupe", "crimson"]);
    }

    #[actix_rt::test]
    async fn test_mock() {
        let mocked_values = vec!["cornflour", "Delta", "California", "elegant", "creatures"]
            .into_iter()
            .enumerate()
            .map(|(idx, v)| TR {
                id: idx as i32,
                col1: String::from(v),
            })
            .collect::<Vec<_>>();
        let mut mock = MockDatasource::new();
        mock.expect_select_all_test()
            .return_once(move || Ok(mocked_values));

        let results = model::get_datas_start_with_char(&mock, 'c').await.unwrap();

        assert_eq!(results, vec!["cornflour", "creatures"]);
    }

    #[actix_rt::test]
    async fn test_mock_error() {
        let mut mock = MockPgDatasource::new();
        mock.expect_select_all_test()
            .return_once(move || Err(sqlx::Error::PoolTimedOut));

        let results = model::get_datas_start_with_char(&mock, 'c')
            .await
            .unwrap_err();

        assert!(matches!(
            results,
            model::ModelError::DatasourceError(sqlx::Error::PoolTimedOut)
        ));
    }
}
