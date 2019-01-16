#![allow(unused_variables)]
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate serde_derive;
extern crate serde_json;
extern crate reqwest;

use actix_web::{App, AsyncResponder, Error, fs, HttpMessage, HttpRequest, HttpResponse, Result, server};
use actix_web::{middleware};
use actix_web::{http};
use actix_web::http::{Method};
use actix_web::client;
use futures::{Future, future::ok as fut_ok};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use bytes;

// simple handle
fn index(req: &HttpRequest) -> Result<HttpResponse, Error> {
    println!("{:?}", req);
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("Welcome!"))
}

fn get_image(req: &HttpRequest) -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("./src/assets/256.png")?)
}

fn get_map_image() -> Box<Future<Item=bytes::Bytes, Error=Error>> {
    let url = "https://a.tile.openstreetmap.org/6/31/23.png";
    Box::new(
        client::ClientRequest::get(url)
        .finish()
        .unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|resp| resp.body()
            .from_err()
            .and_then(|body| {
                fut_ok(body)
            })
        )
    )
}

fn send_map_image(req: &HttpRequest) -> Box<Future<Item=HttpResponse, Error=Error>> {
    get_map_image()
        .and_then(|response| {
            Ok(HttpResponse::Ok()
                .content_type("image/png")
                .body(response))
        }).responder()
}

fn main() {
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
    let sys = actix::System::new("ws-example");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    server::new(|| {
        App::new()
            // enable logger
            .middleware(middleware::Logger::default())
            // register simple handler, handle all methods
            .resource("/index.html", |r| r.f(index))
            // with path parameters
            .resource("/", |r| r.method(http::Method::GET).f(|req| {
                HttpResponse::Found()
                    .header("LOCATION", "/index.html")
                    .finish()
            }))
            .resource("/image", |r| r.f(get_image))
            .resource("/map_image", |r| r.method(Method::GET).a(send_map_image))
    }).bind_ssl("127.0.0.1:8443", builder)
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8443");
    let _ = sys.run();
}
