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
    let arc_db = Arc::new(PgDatasource::new(pool));

    let datasource = Data::from(arc_db.clone() as Arc<dyn Datasource<Error = sqlx::Error>>);
    let auth_datasource =
        Data::from(arc_db.clone() as Arc<dyn auth::AuthDatasource<Error = sqlx::Error>>);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(datasource.clone())
            .app_data(auth_datasource.clone())
            .service(handlers::list_extractor)
            .service(handlers::list_with_limits)
            .service(web::resource("/listr").route(web::get().to(handlers::list_request)))
            .route("/", web::get().to(|| HttpResponse::Ok()))
    })
    .bind(("127.0.0.1", 5990))?
    .run()
    .await
}
