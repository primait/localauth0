use std::time::Duration;

use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};

use auth0local::controller;
use auth0local::model::AppData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data: Data<AppData> = Data::new(AppData::new().expect("Failed to create AppData"));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(controller::jwks)
            .service(controller::jwt)
            .service(controller::set_permissions_for_audience)
    })
    .keep_alive(Duration::from_secs(61))
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
