use std::time::Duration;

use actix_files::{Files, NamedFile};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::{self, Data};
use actix_web::{guard, middleware, App, HttpServer};

use futures::Future;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod};
use prima_rs_logger::GuardLoggerCell;

use localauth0::config::Config;
use localauth0::model::{certificates, AppData};
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

    let http_server = start_http_server(data.clone());
    let https_server = start_https_server(data.clone());

    futures::try_join!(http_server, https_server).map(|_| ())
}

fn start_http_server(data: Data<AppData>) -> impl Future<Output = Result<(), std::io::Error>> {
    let port = *data.config().http().port();
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .configure(setup_service)
    })
    .bind(("0.0.0.0", port))
    .unwrap()
    .keep_alive(Duration::from_secs(61))
    .run()
}

fn start_https_server(data: Data<AppData>) -> impl Future<Output = Result<(), std::io::Error>> {
    let port = *data.config().https().port();
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .configure(setup_service)
    })
    .bind_openssl(("0.0.0.0", port), setup_ssl_acceptor())
    .expect("Cannot bind openssl socket")
    .keep_alive(Duration::from_secs(61))
    .run()
}

fn setup_service(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::jwks)
        .service(controller::get_permissions)
        .service(controller::set_permissions_for_audience)
        .service(controller::get_permissions_by_audience)
        .service(controller::rotate_keys)
        .service(controller::revoke_keys)
        .service(controller::login)
        .service(controller::token)
        .service(
            Files::new("/", "./web/dist")
                .index_file("index.html")
                .default_handler(|req: ServiceRequest| async {
                    let (http_req, _payload) = req.into_parts();
                    let response = NamedFile::open("./web/dist/index.html")?.into_response(&http_req);
                    Ok(ServiceResponse::new(http_req, response))
                }),
        );
}

fn setup_ssl_acceptor() -> SslAcceptorBuilder {
    let pkey = certificates::generate_private_key().expect("Failed to generate the private key");
    let certificate = certificates::generate_certificate(&pkey).expect("Failed to generate the certificate");
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("Failed to create the SSL acceptor");
    builder.set_private_key(&pkey).expect("Error setting the private key");
    builder
        .set_certificate(&certificate)
        .expect("Error setting the certificate");
    builder
}
