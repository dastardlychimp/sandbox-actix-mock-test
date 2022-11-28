use async_trait::async_trait;
use thiserror;
use sqlx::{prelude::*, PgPool, query_as_unchecked};

#[cfg(test)]
use mockall::*;

#[derive(sqlx::FromRow)]
pub struct TR {
    id: i32,
    col1: String,
}

#[async_trait]
trait Datasource {
    type Error;

    async fn select_all_test(&self) -> Result<Vec<TR>, Self::Error>;

    async fn select_last_test(&self) -> Result<TR, Self::Error>;
}

struct Post {
    pool: PgPool
}

#[cfg_attr(test, automock)]
#[async_trait]
impl Datasource for Post {
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


mod model {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    pub enum ModelError<E> {
        #[error("datasource error: {0:?}")]
        DatasourceError(#[from] E)
    }

    pub async fn get_datas_start_with_char<E>(datasource: &dyn Datasource<Error = E>, starting_char: char) -> Result<Vec<String>, ModelError<E>>
    {
        let rows = datasource.select_all_test().await?;

        let filtered_rows = rows.into_iter().filter_map(|row|
            if row.col1.starts_with(starting_char) {
                Some(row.col1)
            } else {
                None
            }
        ).collect::<_>();
        
        Ok(filtered_rows)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{test};


    #[actix_web::test]
    async fn test_pg() {
        let pool = PgPool::connect(env!("DATABASE_URL")).await.unwrap();
        let db = Post { pool };
        let results = db.select_all_test().await.unwrap();

        assert!(results.len() > 1)
    }

    #[actix_web::test]
    async fn test_all_rows_with_char() {
        let pool = PgPool::connect(env!("DATABASE_URL")).await.unwrap();
        let db = Post { pool };
        let results = model::get_datas_start_with_char(&db, 'c')
            .await
            .unwrap();

        assert_eq!(results, vec!["c", "cantaloupe", "crimson"]);
    }


    #[actix_web::test]
    async fn test_mock() {
        let mocked_values = vec!["cornflour", "Delta", "California", "elegant", "creatures"]
            .into_iter()
            .enumerate()
            .map(|(idx, v)| TR {
                id: idx as i32,
                col1: String::from(v),
            })
            .collect::<Vec<_>>();
        let mut mock = MockPost::new();
        mock.expect_select_all_test().return_once(move || Ok(mocked_values));

        let results = model::get_datas_start_with_char(&mock, 'c').await.unwrap();

        assert_eq!(results, vec!["cornflour", "creatures"]);
    }

    #[actix_web::test]
    async fn test_mock_error() {
        let mut mock = MockPost::new();
        mock.expect_select_all_test().return_once(move || Err(sqlx::Error::PoolTimedOut));

        let results = model::get_datas_start_with_char(&mock, 'c').await.unwrap_err();

        assert!(matches!(results, model::ModelError::DatasourceError(sqlx::Error::PoolTimedOut)));
    }

}