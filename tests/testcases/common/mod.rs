extern crate futures;
extern crate rand;
extern crate stq_http;
extern crate stq_logging;
extern crate tokio_core;

extern crate warehouses_lib as lib;

use self::rand::Rng;
use std::sync::mpsc::channel;
use std::thread;

pub fn setup() -> String {
    stq_logging::init(None);

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

    format!("http://localhost:{}", port)
}
