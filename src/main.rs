extern crate stq_logging;
extern crate warehouses_lib as lib;

fn main() {
    let config = lib::Config::new()
        .expect("Failed to load service configuration. Please check your 'config' folder");

    // Prepare sentry integration
    let _sentry = lib::sentry_integration::init(config.sentry.as_ref());

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    lib::start_server(config, None, || ());
}
