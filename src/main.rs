use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use env_logger;
use sandbox::*;
use sqlx::PgPool;

use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let pool = PgPool::connect(env!("DATABASE_URL")).await.unwrap();
    let db: Arc<dyn Datasource<Error = sqlx::Error> + Send + Sync> =
        Arc::new(PgDatasource::new(pool));
    let db_data = Data::from(db);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(db_data.clone())
            .service(handlers::list_extractor)
            .service(web::resource("/listr").route(web::get().to(handlers::list_request)))
            .route("/", web::get().to(|| HttpResponse::Ok()))
    })
    .bind(("127.0.0.1", 5990))?
    .run()
    .await
}
