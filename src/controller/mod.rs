use futures::future;
use futures::prelude::*;
use hyper::{Delete, Get, Post, Put, Request};
use std::sync::Arc;

use stq_http::controller::Controller;
use stq_http::errors::ControllerError;
use stq_http::request_util::{parse_body, serialize_future, ControllerFuture};
use stq_router::RouteParser;

use config::*;
use models::*;
use repos;
use services::*;
pub mod routing;
use self::routing::*;
use types::*;

pub struct ControllerImpl {
    route_parser: Arc<RouteParser<Route>>,
    service_factory: Arc<Fn() -> Box<WarehouseService>>,
}

impl ControllerImpl {
    pub fn new(db_pool: DbPool, _config: Config) -> Self {
        ControllerImpl {
            service_factory: Arc::new({
                let db_pool = db_pool.clone();
                move || {
                    Box::new(WarehouseServiceImpl {
                        db_pool: db_pool.clone(),
                        repo_factory: RepoFactory {
                            warehouse_repo_factory: Arc::new(|| {
                                Box::new(repos::warehouse::make_repo())
                            }),
                            warehouse_products_repo_factory: Arc::new(|| {
                                Box::new(repos::warehouse_products::make_repo())
                            }),
                            warehouse_users_repo_factory: Arc::new(|| {
                                Box::new(repos::warehouse_users::make_repo())
                            }),
                        },
                    })
                }
            }),
            route_parser: Arc::new(routing::make_router()),
        }
    }
}

impl Controller for ControllerImpl {
    fn call(&self, request: Request) -> ControllerFuture {
        let (method, uri, _, _, payload) = request.deconstruct();

        let service_factory = self.service_factory.clone();
        let route_parser = self.route_parser.clone();

        let route = route_parser.test(uri.path());

        match (method, route) {
            (Get, Some(Route::Warehouse { warehouse_id })) => serialize_future({
                debug!("Received request to get warehouse {}", warehouse_id.0);
                (service_factory)()
                    .get_warehouse(warehouse_id)
                    .map_err(ControllerError::InternalServerError)
            }),
            (Post, Some(Route::Warehouses)) => serialize_future({
                debug!("Received request to create warehouse");
                parse_body::<WarehouseInserter>(payload).and_then(move |data| {
                    (service_factory)()
                        .create_warehouse(data)
                        .map_err(ControllerError::InternalServerError)
                })
            }),
            (Put, Some(Route::Warehouse { warehouse_id })) => serialize_future({
                debug!("Received request to update warehouse {}", warehouse_id.0);
                parse_body::<WarehouseUpdateData>(payload).and_then(move |data| {
                    (service_factory)()
                        .update_warehouse(warehouse_id, data)
                        .map_err(ControllerError::InternalServerError)
                })
            }),
            (Delete, Some(Route::Warehouse { warehouse_id })) => serialize_future({
                debug!("Received request to delete warehouse {}", warehouse_id.0);
                (service_factory)()
                    .delete_warehouse(warehouse_id)
                    .map_err(ControllerError::InternalServerError)
            }),
            (Post, Some(Route::DeleteAllWarehouses)) => serialize_future({
                debug!("Received request to delete all warehouses");
                (service_factory)()
                    .delete_all_warehouses()
                    .map_err(ControllerError::InternalServerError)
            }),
            (Get, Some(Route::ProductsInWarehouse { warehouse_id })) => serialize_future({
                debug!(
                    "Received request to get products of warehouse {}",
                    warehouse_id.0
                );
                (service_factory)()
                    .list_products_in_warehouse(warehouse_id)
                    .map_err(ControllerError::InternalServerError)
            }),
            (
                Get,
                Some(Route::ProductInWarehouse {
                    warehouse_id,
                    product_id,
                }),
            ) => serialize_future({
                debug!(
                    "Received request to get product {} of warehouse {}",
                    product_id.0, warehouse_id.0
                );
                (service_factory)()
                    .get_product_in_warehouse(warehouse_id, product_id)
                    .map_err(ControllerError::InternalServerError)
            }),
            (
                Post,
                Some(Route::ProductInWarehouse {
                    warehouse_id,
                    product_id,
                }),
            ) => serialize_future({
                debug!(
                    "Received request to add product {} to warehouse {}",
                    product_id.0, warehouse_id.0
                );
                (service_factory)()
                    .add_product_to_warehouse(warehouse_id, product_id)
                    .map_err(ControllerError::InternalServerError)
            }),
            (
                Put,
                Some(Route::ProductInWarehouse {
                    warehouse_id,
                    product_id,
                }),
            ) => serialize_future({
                parse_body::<WarehouseProductUpdateData>(payload).and_then(move |data| {
                    debug!(
                        "Received request to update product {} in warehouse {} with the following data {:?}",
                        product_id.0, warehouse_id.0, &data
                    );
                    (service_factory)()
                        .update_product_in_warehouse(warehouse_id, product_id, data)
                        .map_err(ControllerError::InternalServerError)
                })
            }),
            (Get, Some(Route::Product { product_id })) => serialize_future({
                debug!(
                    "Received request to get product {} in all warehouses",
                    product_id.0
                );
                (service_factory)()
                    .find_by_product_id(product_id)
                    .map_err(ControllerError::InternalServerError)
            }),
            // Fallback
            _ => Box::new(future::err(ControllerError::NotFound)),
        }
    }
}
