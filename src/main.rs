mod utils;
mod weekly;

use crate::weekly::weekly;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use std::{env, io::Result, net::SocketAddr};

#[macro_use]
extern crate log;

async fn root() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("index.html"))
}

#[actix_rt::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let port: u16 = match env::var("PORT") {
        Ok(port_str) => port_str.parse().expect("Could not use provided port."),
        Err(_) => 4343,
    };

    let app = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("[RETURNED HTTP %s] [TOOK %Dms]"))
            .route("/", web::get().to(root))
            .route("/{autologin}/weekly.ics", web::get().to(weekly))
    });

    info!("starting server on http://localhost:{}", port);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    app.bind(addr)?.run().await
}
