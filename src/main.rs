use std::error::Error;
use std::time::Duration;

use actix_files::{Files, NamedFile};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::web::{self, Data};
use actix_web::{middleware, App, HttpServer};

use futures::Future;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod};

use localauth0::config::Config;
use localauth0::controller;
use localauth0::model::{certificates, AppData};

fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("healthcheck") => Ok(healthcheck()?),
        _ => Ok(server()?),
    }
}

async fn is_endpoint_healthy(client: &reqwest::Client, endpoint: String) -> bool {
    let status = client.get(&endpoint).send().await.map(|r| r.error_for_status());

    match status {
        Ok(Ok(_)) => {
            println!("{endpoint} OK");
            true
        }
        _ => {
            println!("{endpoint} ERROR {status:?}");
            false
        }
    }
}

fn healthcheck() -> Result<(), String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let config = Config::load_or_default();

            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();

            let endpoint_http = format!("http://127.0.0.1:{}/check", config.http().port());
            let endpoint_https = format!("https://127.0.0.1:{}/check", config.https().port());

            let http_healthy = is_endpoint_healthy(&client, endpoint_http).await;
            let https_healthy = is_endpoint_healthy(&client, endpoint_https).await;

            if http_healthy && https_healthy {
                Ok(())
            } else {
                Err("healthcheck failed".to_string())
            }
        })
}

#[actix_web::main]
async fn server() -> std::io::Result<()> {
    let config = Config::load_or_default();
    let data: Data<AppData> = Data::new(AppData::new(config).expect("Failed to create AppData"));

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let http_server = start_http_server(data.clone());
    let https_server = start_https_server(data.clone());

    futures::try_join!(http_server, https_server).map(|_| ())
}

fn start_http_server(data: Data<AppData>) -> impl Future<Output = Result<(), std::io::Error>> {
    let port = *data.config().http().port();
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default().exclude("/check"))
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
            .wrap(middleware::Logger::default().exclude("/check"))
            .configure(setup_service)
    })
    .bind_openssl(("0.0.0.0", port), setup_ssl_acceptor())
    .expect("Cannot bind openssl socket")
    .keep_alive(Duration::from_secs(61))
    .run()
}

fn setup_service(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::healthcheck)
        .service(controller::jwks)
        .service(controller::get_permissions)
        .service(controller::set_permissions_for_audience)
        .service(controller::get_permissions_by_audience)
        .service(controller::rotate_keys)
        .service(controller::revoke_keys)
        .service(controller::login)
        .service(controller::token)
        .service(controller::openid_configuration)
        .service(
            Files::new("/", "./web")
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
