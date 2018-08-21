use stq_api::warehouses::*;
use stq_roles;
use stq_router::*;
use stq_types::*;

pub fn make_router() -> RouteParser<Route> {
    let mut route_parser: RouteParser<Route> = Default::default();

    route_parser.add_route(r"^/warehouses$", || Route::Warehouses);
    route_parser.add_route_with_params(r"^/warehouses/by-id/([a-zA-Z0-9-]+)/products$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|warehouse_id| Route::StocksInWarehouse { warehouse_id })
    });
    route_parser.add_route_with_params(
        r"^/warehouses/by-id/([a-zA-Z0-9-]+)/products/(\d+)$",
        |params| {
            if let Some(warehouse_id_s) = params.get(0) {
                if let Some(product_id_s) = params.get(1) {
                    if let Ok(warehouse_id) = warehouse_id_s.parse().map(WarehouseId) {
                        if let Ok(product_id) = product_id_s.parse().map(ProductId) {
                            return Some(Route::StockInWarehouse {
                                warehouse_id,
                                product_id,
                            });
                        }
                    }
                }
            }
            None
        },
    );
    route_parser.add_route_with_params(r"^/warehouses/by-id/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok().map(WarehouseIdentifier::Id))
            .map(|warehouse_id| Route::Warehouse { warehouse_id })
    });
    route_parser.add_route_with_params(r"^/warehouses/by-slug/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok().map(WarehouseIdentifier::Slug))
            .map(|warehouse_id| Route::Warehouse { warehouse_id })
    });
    route_parser.add_route_with_params(r"^/warehouses/by-store/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|store_id| Route::WarehousesByStore { store_id })
    });

    route_parser.add_route_with_params(r"^/stocks/by-product-id/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|product_id| Route::StocksByProductId { product_id })
    });
    route_parser.add_route_with_params(r"^/stocks/by-id/([a-zA-Z0-9-]+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok())
            .map(|stock_id| Route::StockById { stock_id })
    });

    route_parser = stq_roles::routing::add_routes(route_parser);

    route_parser
}
