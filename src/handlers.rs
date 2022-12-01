use crate::*;
use actix_web::{
    web::{self, Data, Json, Query},
    FromRequest, HttpRequest, Responder, Result as ActixResult,
};
use auth::*;

#[actix_web::get("/list")]
pub async fn list_extractor(
    datasource: Data<dyn Datasource<Error = sqlx::Error> >,
) -> impl Responder {
    let results = datasource.select_all_test().await.unwrap();
    web::Json(results)
}

pub async fn list_request(request: HttpRequest) -> ActixResult<Json<Vec<TR>>> {
    let datasource =
        Data::<dyn Datasource<Error = sqlx::Error> >::extract(&request).await?;
    let results = datasource.select_all_test().await.unwrap();

    Ok(web::Json(results))
}

pub async fn list_generic<D: Datasource<Error = E>, E: std::fmt::Debug>(
    datasource: Data<D>
) -> web::Json<Vec<TR>> {
    let results = datasource.select_all_test().await.unwrap();
    web::Json(results)
}

#[derive(Deserialize)]
pub struct ListWithLimitsQuery {
    key: String
}

#[actix_web::get("/listl")]
pub async fn list_with_limits(
    datasource: Data<dyn Datasource<Error = sqlx::Error>>,
    auth_datasource: Data<dyn AuthDatasource<Error = sqlx::Error>>,
    query: Query<ListWithLimitsQuery>,
) -> ActixResult<impl Responder> {
    let limit = auth_datasource.key_limit(&query.key)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .ok_or(actix_web::error::ErrorUnauthorized("invalid key"))?;
    
    let mut results = datasource.select_all_test().await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let auth::KeyLimit::Limit(l) = limit {
        results.truncate(l);
    }

    return Ok(web::Json(results));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockPgDatasource, auth::AuthDatasource};
    use actix_web::{test::{self, TestRequest}, App, http::StatusCode};
    use std::sync::Arc;
    use mockall::predicate::*;

    #[actix_rt::test]
    async fn test_mock_list_request() {
        let expected = vec![TR {
            id: 6,
            col1: "wyoming".to_string(),
        }];
        let expected2 = expected.clone();
        let mut mock = MockPgDatasource::new();
        mock.expect_select_all_test()
            .returning(move || Ok(expected2.to_owned()));

        let arc_data: Arc<dyn Datasource<Error = sqlx::Error> > = Arc::new(mock);
        let data: Data<dyn Datasource<Error = sqlx::Error> > = Data::from(arc_data);

        let req = test::TestRequest::default()
            .app_data(data)
            .to_http_request();

        let resp = list_request(req.clone()).await.unwrap();
        assert_eq!(resp.into_inner(), expected);
    }

    #[actix_rt::test]
    async fn test_mock_list_generic() {
        let expected = vec![TR {
            id: 6,
            col1: "wyoming".to_string(),
        }];
        let expected2 = expected.clone();
        let mut mock = MockPgDatasource::new();
        mock.expect_select_all_test()
            .returning(move || Ok(expected2.to_owned()));


        let resp: web::Json<Vec<TR>> = list_generic(Data::new(mock)).await;
        assert_eq!(resp.into_inner(), expected);
    }

    // Integration style can use extractors
    #[actix_rt::test]
    async fn test_mock_list_extractor() {
        let expected = vec![TR {
            id: 6,
            col1: "wyoming".to_string(),
        }];
        let expected2 = expected.clone();
        let mut mock = MockPgDatasource::new();
        mock.expect_select_all_test()
            .returning(move || Ok(expected2.to_owned()));

        let arc_data: Arc<dyn Datasource<Error = sqlx::Error> > = Arc::new(mock);
        let data = Data::from(arc_data);

        let mut app = test::init_service(App::new().service(list_extractor).app_data(data)).await;

        let req = test::TestRequest::default().uri("/list").to_request();

        let resp = test::call_service(&mut app, req).await;
        let body = test::read_body_json::<Vec<TR>, _>(resp).await;
        assert_eq!(body, expected);
    }
    
    // Integration style can use extractors
    #[actix_rt::test]
    async fn test_mock_list_limited() {
        let mut mock = MockPgDatasource::new();
        let it = std::iter::repeat(TR {id: -1, col1: "repeat".to_string()});
        let cb_gen = move || it.clone().take(50).collect::<Vec<_>>();

        mock.expect_select_all_test().returning(move || Ok(cb_gen()));
        mock.expect_key_limit()
            .with(eq("works"))
            .returning(|_| Ok(Some(auth::KeyLimit::Limit(10))));
        mock.expect_key_limit()
            .with(eq(UNLIMITED_KEY))
            .returning(|_| Ok(Some(auth::KeyLimit::Unlimited)));
        mock.expect_key_limit()
            .returning(|_| Ok(None));

        let arc_mock = Arc::new(mock);

        let mock_datasource = {
            let arc_data: Arc<dyn Datasource<Error = sqlx::Error> > = arc_mock.clone();
            Data::from(arc_data)
        };

        let auth_datasource = {
            let arc_data: Arc<dyn AuthDatasource<Error = sqlx::Error> > = arc_mock.clone();
            Data::from(arc_data)
        };

        let mut app = test::init_service(
            App::new()
                .service(list_with_limits)
                .app_data(mock_datasource)
                .app_data(auth_datasource)
        ).await;

        {
            let key = "noworks";
            let req = test::TestRequest::default().uri(&format!("/listl?key={}", key)).to_request();
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
        {
            let key = "works";
            let req = test::TestRequest::default().uri(&format!("/listl?key={}", key)).to_request();
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body = test::read_body_json::<Vec<TR>, _>(resp).await;
            assert_eq!(body.len(), 10);
        }
        {
            let key = auth::UNLIMITED_KEY;
            let req = test::TestRequest::default().uri(&format!("/listl?key={}", key)).to_request();
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
            let body = test::read_body_json::<Vec<TR>, _>(resp).await;
            assert_eq!(body.len(), 50);
        }
    }
}
