use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use dotenv::dotenv;
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use log::{debug, info, trace, warn};
use serde_derive::Deserialize;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;

#[derive(Deserialize)]
struct Config {
    address: SocketAddr,
}

fn main() {
    pretty_env_logger::init();
    // 1. Read .env file to load env
    // dotenv().ok();
    // 2. Set env with command line
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .value_name("ADDRESS")
                .help("Sets an address")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    // 3. Read env from TOML file
    let config = File::open("microservice.toml")
        .and_then(|mut file| {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            Ok(buffer)
        })
        .and_then(|buffer| {
            toml::from_str::<Config>(&buffer)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
        })
        .map_err(|err| warn!("Can't read config file: {}", err))
        .ok();

    info!("Rand Microservice - v0.1.0");
    trace!("Starting....");
    // This takes ADDRESS env var from the command line
    let addr = matches
        .value_of("address") // From cli
        .map(|s| s.to_owned())
        .or(env::var("ADDRESS").ok()) // From env var
        .and_then(|addr| addr.parse().ok())
        .or(config.map(|config| config.address)) // From toml file
        .or_else(|| Some(([127, 0, 0, 1], 8080).into())) // Use default
        .unwrap();
    let builder = Server::bind(&addr);
    trace!("Creating service handler...");
    let server = builder.serve(|| {
        service_fn_ok(|req| {
            trace!("Incoming request is: {:?}", req);
            let random_byte = rand::random::<u8>();
            debug!("Generated value is: {}", random_byte);
            Response::new(Body::from(random_byte.to_string()))
        })
    });
    info!("Used address: {}", server.local_addr());
    let server = server.map_err(drop);
    debug!("Run!");
    hyper::rt::run(server);
}
