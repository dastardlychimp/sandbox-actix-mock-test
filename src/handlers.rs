use crate::*;
use actix_web::{
    web::{self, Data, Json},
    FromRequest, HttpRequest, Responder, Result as ActixResult,
};

#[actix_web::get("/list")]
pub async fn list_extractor(
    datasource: Data<dyn Datasource<Error = sqlx::Error> + Send + Sync>,
) -> impl Responder {
    let results = datasource.select_all_test().await.unwrap();
    web::Json(results)
}

pub async fn list_request(request: HttpRequest) -> ActixResult<Json<Vec<TR>>> {
    let datasource =
        Data::<dyn Datasource<Error = sqlx::Error> + Send + Sync>::extract(&request).await?;
    let results = datasource.select_all_test().await.unwrap();

    Ok(web::Json(results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockPgDatasource;
    use actix_web::{test, App};
    use std::sync::Arc;

    #[actix_web::test]
    async fn test_mock_list_request() {
        let expected = vec![TR {
            id: 6,
            col1: "wyoming".to_string(),
        }];
        let expected2 = expected.clone();
        let mut mock = MockPgDatasource::new();
        mock.expect_select_all_test()
            .returning(move || Ok(expected2.to_owned()));

        let arc_data: Arc<dyn Datasource<Error = sqlx::Error> + Send + Sync> = Arc::new(mock);
        let data: Data<dyn Datasource<Error = sqlx::Error> + Send + Sync> = Data::from(arc_data);

        let req = test::TestRequest::default()
            .app_data(data)
            .to_http_request();

        let resp = list_request(req.clone()).await.unwrap();
        assert_eq!(resp.into_inner(), expected);
    }

    // Integration style can use extractors
    #[actix_web::test]
    async fn test_mock_list_extractor() {
        let expected = vec![TR {
            id: 6,
            col1: "wyoming".to_string(),
        }];
        let expected2 = expected.clone();
        let mut mock = MockPgDatasource::new();
        mock.expect_select_all_test()
            .returning(move || Ok(expected2.to_owned()));

        let arc_data: Arc<dyn Datasource<Error = sqlx::Error> + Send + Sync> = Arc::new(mock);
        let data = Data::from(arc_data);

        let app = test::init_service(App::new().service(list_extractor).app_data(data)).await;

        let req = test::TestRequest::default().uri("/list").to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body_json::<Vec<TR>, _>(resp).await;
        assert_eq!(body, expected);
    }
}
