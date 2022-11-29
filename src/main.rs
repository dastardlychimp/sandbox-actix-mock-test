use actix_web::{web::{self, Data}, middleware::Logger, App, HttpServer, HttpResponse};
use env_logger;
use sandbox::*;
use sqlx::PgPool;

use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let pool = PgPool::connect(env!("DATABASE_URL")).await.unwrap();
    let db: Arc<dyn Datasource<Error = sqlx::Error> + Send + Sync> = Arc::new(PgDatasource::new(pool));
    let db_data = Data::from(db);
    HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
        .app_data(db_data.clone())
        .service(list)
        .route("/", web::get().to(|| HttpResponse::Ok()))
    })
    .bind(("127.0.0.1", 5990))?
    .run()
    .await
}

// #[actix_web::get("/list")]
// async fn list(datasource: Data<PgDatasource>) -> impl actix_web::Responder {
//     let results = datasource.select_all_test().await.unwrap();
//     web::Json(results)
// }

#[actix_web::get("/list")]
async fn list(datasource: Data<dyn Datasource<Error = sqlx::Error> + Send + Sync>) -> impl actix_web::Responder {
    let results = datasource.select_all_test().await.unwrap();
    web::Json(results)
}
