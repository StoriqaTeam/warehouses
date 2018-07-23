extern crate warehouses_lib as lib;
extern crate stq_logging;

fn main() {
    let config = lib::Config::new()
        .expect("Failed to load service configuration. Please check your 'config' folder");

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    lib::start_server(config, None, || ());
}
