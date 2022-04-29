use std::time::Duration;

use actix_files::Files;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};

use localauth0::config::Config;
use localauth0::controller;
use localauth0::model::AppData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv::dotenv().expect("Failed to load .env file");
    let config: Config = Config::load().expect("Failed to load configuration");

    let data: Data<AppData> = Data::new(AppData::new().expect("Failed to create AppData"));

    data.audience().load_permission_settings(config.permission_settings());

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(controller::jwks)
            .service(controller::jwt)
            .service(controller::set_permissions_for_audience)
            .service(controller::get_permissions_by_audience)
            .service(controller::rotate_keys)
            .service(controller::revoke_keys)
            .service(Files::new("/", "./web/dist/").index_file("index.html"))
    })
    .keep_alive(Duration::from_secs(61))
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
