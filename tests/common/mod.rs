extern crate futures;
extern crate rand;
extern crate stq_http;
extern crate tokio_core;

extern crate warehouses_lib as lib;

use self::futures::prelude::*;
use self::rand::Rng;
use self::stq_http::client::{
    Client as HttpClient, ClientHandle as HttpClientHandle, Config as HttpConfig,
};
use self::tokio_core::reactor::Core;
use std::sync::mpsc::channel;
use std::thread;

pub struct Context {
    pub http_client: HttpClientHandle,
    pub base_url: String,
    pub core: Core,
}

pub fn setup() -> Context {
    let (tx, rx) = channel::<bool>();
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(50000, 60000);
    thread::spawn({
        let tx = tx.clone();
        move || {
            let config = lib::Config::new().expect("Can't load app config!");
            lib::start_server(config, Some(port), move || {
                let _ = tx.send(true);
            });
        }
    });
    rx.recv().unwrap();
    let core = Core::new().expect("Unexpected error creating event loop core");
    let client = HttpClient::new(
        &HttpConfig {
            http_client_retries: 3,
            http_client_buffer_size: 3,
        },
        &core.handle(),
    );
    let client_handle = client.handle();
    core.handle().spawn(client.stream().for_each(|_| Ok(())));

    Context {
        http_client: client_handle,
        base_url: format!("http://localhost:{}", port),
        core,
    }
}
