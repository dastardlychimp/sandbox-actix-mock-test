#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use actix_web::{
    web::{self, Data},
    middleware::Logger, App, HttpServer, HttpResponse,
};
use env_logger;
use sandbox::*;
use sqlx::PgPool;
use std::sync::Arc;
fn main() -> std::io::Result<()> {
    <::actix_web::rt::System>::new()
        .block_on(async move {
            {
                env_logger::init_from_env(
                    env_logger::Env::new().default_filter_or("debug"),
                );
                let pool = PgPool::connect("postgres://postgres@localhost:5995/sandbox")
                    .await
                    .unwrap();
                let db: Arc<dyn Datasource<Error = sqlx::Error> + Send + Sync> = Arc::new(
                    PgDatasource::new(pool),
                );
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
        })
}
#[allow(non_camel_case_types, missing_docs)]
pub struct list;
impl ::actix_web::dev::HttpServiceFactory for list {
    fn register(self, __config: &mut actix_web::dev::AppService) {
        async fn list(
            datasource: Data<dyn Datasource<Error = sqlx::Error> + Send + Sync>,
        ) -> impl actix_web::Responder {
            let results = datasource.select_all_test().await.unwrap();
            web::Json(results)
        }
        let __resource = ::actix_web::Resource::new("/list")
            .name("list")
            .guard(::actix_web::guard::Get())
            .to(list);
        ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
    }
}
