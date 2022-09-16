use std::time::Duration;

use actix_files::{Files, NamedFile};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::{self, Data};
use actix_web::{guard, middleware, App, HttpServer};
use prima_rs_logger::GuardLoggerCell;

use localauth0::config::Config;
use localauth0::model::AppData;
use localauth0::{controller, APP_NAME};

// Singleton logger. Used to free user from manually passing Logger objects around.
static LOGGER_GUARD: GuardLoggerCell = GuardLoggerCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    LOGGER_GUARD
        .set(prima_rs_logger::term_guard(APP_NAME))
        .expect("Cannot set global logger guard");

    let config: Config = Config::load();
    let data: Data<AppData> = Data::new(AppData::new(config).expect("Failed to create AppData"));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(controller::jwks)
            .service(controller::get_permissions)
            .service(controller::set_permissions_for_audience)
            .service(controller::get_permissions_by_audience)
            .service(controller::rotate_keys)
            .service(controller::revoke_keys)
            .service(controller::login)
            .service(
                web::resource("/oauth/token")
                    .route(
                        web::post()
                            .guard(guard::Header("content-type", "application/json"))
                            .to(controller::jwt_json_body_handler),
                    )
                    .route(
                        web::post()
                            .guard(guard::Header("content-type", "application/x-www-form-urlencoded"))
                            .to(controller::jwt_form_body_handler),
                    ),
            )
            .service(Files::new("/", "./web/dist").index_file("index.html").default_handler(
                |req: ServiceRequest| async {
                    let (http_req, _payload) = req.into_parts();
                    let response = NamedFile::open("./web/dist/index.html")?.into_response(&http_req);
                    Ok(ServiceResponse::new(http_req, response))
                },
            ))
    })
    .keep_alive(Duration::from_secs(61))
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
