use models::*;

use stq_router::*;

#[derive(Clone, Copy, Debug)]
pub enum Route {
    Warehouses,
    Warehouse {
        warehouse_id: WarehouseId,
    },
    ProductsInWarehouse {
        warehouse_id: WarehouseId,
    },
    ProductInWarehouse {
        warehouse_id: WarehouseId,
        product_id: ProductId,
    },
    DeleteAllWarehouses,
    Product {
        product_id: ProductId,
    },
    WarehouseProduct {
        warehouse_product_id: WarehouseProductId,
    },
}

pub fn make_router() -> RouteParser<Route> {
    let mut route_parser: RouteParser<Route> = Default::default();
    route_parser.add_route(r"^/warehouses$", || Route::Warehouses);
    route_parser.add_route_with_params(r"^/warehouses/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok().map(WarehouseId))
            .map(|warehouse_id| Route::Warehouse { warehouse_id })
    });
    route_parser.add_route_with_params(r"^/warehouses/(\d+)/products$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok().map(WarehouseId))
            .map(|warehouse_id| Route::ProductsInWarehouse { warehouse_id })
    });
    route_parser.add_route_with_params(r"^/warehouses/(\d+)/products/(\d+)$", |params| {
        if let Some(warehouse_id_s) = params.get(0) {
            if let Some(product_id_s) = params.get(1) {
                if let Ok(warehouse_id) = warehouse_id_s.parse() {
                    let warehouse_id = WarehouseId(warehouse_id);
                    if let Ok(product_id) = product_id_s.parse() {
                        let product_id = ProductId(product_id);
                        return Some(Route::ProductInWarehouse {
                            warehouse_id,
                            product_id,
                        });
                    }
                }
            }
        }
        None
    });
    route_parser.add_route(r"^/warehouses/clear$", || Route::DeleteAllWarehouses);
    route_parser.add_route_with_params(r"^/products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok().map(ProductId))
            .map(|product_id| Route::Product { product_id })
    });
    route_parser.add_route_with_params(r"^/warehouse_products/(\d+)$", |params| {
        params
            .get(0)
            .and_then(|string_id| string_id.parse().ok().map(WarehouseProductId))
            .map(|warehouse_product_id| Route::WarehouseProduct {
                warehouse_product_id,
            })
    });

    route_parser
}
