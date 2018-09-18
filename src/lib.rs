extern crate bb8;
extern crate bb8_postgres;
extern crate chrono;
extern crate config as config_crate;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate geo;
extern crate hyper;
extern crate iso_country;
#[macro_use]
extern crate log as log_crate;
extern crate postgres;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate stq_acl;
extern crate stq_api;
extern crate stq_db;
extern crate stq_http;
extern crate stq_logging;
extern crate stq_roles;
extern crate stq_router;
extern crate stq_types;
extern crate tokio_core;
extern crate tokio_postgres;
extern crate uuid;
#[macro_use]
extern crate sentry;

use bb8_postgres::PostgresConnectionManager;
use futures::future;
use futures::prelude::*;
use hyper::server::Http;
use std::net::SocketAddr;
use std::process::exit;
use stq_http::controller::Application;
use tokio_core::reactor::Core;
use tokio_postgres::TlsMode;

mod config;
pub mod controller;
pub mod errors;
pub mod models;
pub mod repos;
pub mod sentry_integration;
pub mod services;
pub mod types;

pub use config::*;

/// Starts web server with the provided configuration
pub fn start_server<F: FnOnce() + 'static>(config: config::Config, port: Option<u16>, callback: F) {
    let mut core = Core::new().expect("Unexpected error creating event loop core");

    let manager = PostgresConnectionManager::new(config.db.dsn.clone(), || TlsMode::None).unwrap();
    let db_pool = {
        let remote = core.remote();
        stq_db::pool::Pool::from(
            core.run(
                bb8::Pool::builder()
                    .max_size(50)
                    .build(manager, remote)
                    .map_err(|e| format_err!("{}", e)),
            ).expect("Failed to create connection pool"),
        )
    };

    let listen_address = {
        let port = port.unwrap_or(config.listen.port);
        SocketAddr::new(config.listen.host, port)
    };

    let serve = Http::new()
        .serve_addr_handle(&listen_address, &core.handle(), move || {
            let controller = controller::ControllerImpl::new(db_pool.clone(), &config);

            // Prepare application
            let app = Application::<errors::Error>::new(controller);

            Ok(app)
        }).unwrap_or_else(|why| {
            error!("Http Server Initialization Error: {}", why);
            exit(1);
        });

    let handle = core.handle();
    handle.spawn(
        serve
            .for_each({
                let handle = handle.clone();
                move |conn| {
                    handle.spawn(
                        conn.map(|_| ())
                            .map_err(|why| error!("Server Error: {:?}", why)),
                    );
                    Ok(())
                }
            }).map_err(|_| ()),
    );

    info!("Listening on http://{}", listen_address);
    handle.spawn_fn(move || {
        callback();
        future::ok(())
    });
    core.run(future::empty::<(), ()>()).unwrap();
}
