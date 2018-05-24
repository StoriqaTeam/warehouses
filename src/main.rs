extern crate warehouses_lib as lib;

fn main() {
    // Prepare logger
    lib::log::log_environment().init();

    let config = lib::Config::new()
        .expect("Failed to load service configuration. Please check your 'config' folder");
    lib::start_server(config, None, || ());
}
