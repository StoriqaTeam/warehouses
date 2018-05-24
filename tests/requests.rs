extern crate futures;
extern crate hyper;
#[macro_use]
extern crate maplit;
extern crate serde_json;
extern crate stq_http;
extern crate tokio_core;
extern crate warehouses_lib as lib;

pub mod common;

use hyper::Method;
use lib::models::*;

#[test]
fn test_warehouses_service() {
    let common::Context {
        mut core,
        http_client,
        base_url,
    } = common::setup();

    core.run(http_client.request_with_auth_header::<Vec<Warehouse>>(
        Method::Post,
        format!("{}/warehouses/clear", base_url),
        None,
        None,
    )).unwrap();

    let mut warehouse = {
        let inserter = WarehouseInserter::new(WarehouseKind::DistributionCenter);

        let res = core.run(http_client.request_with_auth_header::<Option<Warehouse>>(
            Method::Post,
            format!("{}/warehouses", base_url),
            Some(serde_json::to_string(&inserter).unwrap()),
            None,
        )).unwrap()
            .unwrap();

        assert_eq!(
            <(WarehouseId, WarehouseMeta)>::from(res.clone()).1,
            inserter
        );

        res
    };

    {
        let updater = WarehouseUpdateData {
            name: Some(Some("My warehouse".to_string()).into()),
            ..Default::default()
        };

        let res = core.run(http_client.request_with_auth_header::<Option<Warehouse>>(
            Method::Put,
            format!("{}/warehouses/{}", base_url, warehouse.id.0),
            Some(serde_json::to_string(&updater).unwrap()),
            None,
        )).unwrap();

        warehouse.name = Some("My warehouse".to_string());

        assert_eq!(Some(warehouse.clone()), res);
    }

    let mut warehouse_product = {
        let new_product_id = ProductId(2341241);

        let res = core.run(http_client.request_with_auth_header::<WarehouseProduct>(
            Method::Post,
            format!(
                "{}/warehouses/{}/products/{}",
                base_url, warehouse.id.0, new_product_id.0
            ),
            None,
            None,
        )).unwrap();

        let (id, _warehouse_id, _product_id, _meta) = <(
            WarehouseProductId,
            WarehouseId,
            ProductId,
            WarehouseProductMeta,
        )>::from(res.clone());

        let expectation = WarehouseProduct {
            id,
            warehouse_id: warehouse.id,
            product_id: new_product_id,
            quantity: Quantity(0),
        };

        assert_eq!(expectation, res.clone());

        res
    };

    {
        let update_data = WarehouseProductUpdateData {
            quantity: Some(Quantity(4433).into()),
        };

        let res = core.run(http_client.request_with_auth_header::<WarehouseProduct>(
            Method::Put,
            format!(
                "{}/warehouses/{}/products/{}",
                base_url, warehouse.id.0, warehouse_product.product_id.0
            ),
            Some(serde_json::to_string(&update_data).unwrap()),
            None,
        )).unwrap();

        warehouse_product.quantity = update_data.quantity.unwrap().value;

        let expectation = warehouse_product.clone();

        assert_eq!(expectation, res.clone());
    }

    {
        let (_id, _warehouse_id, product_id, meta) = <(
            WarehouseProductId,
            WarehouseId,
            ProductId,
            WarehouseProductMeta,
        )>::from(warehouse_product.clone());

        let expectation = hashmap! {
            product_id => meta,
        };
        let result = core.run(http_client.request_with_auth_header::<WarehouseProductMap>(
            Method::Get,
            format!(
                "{}/warehouses/{}/products",
                base_url, warehouse_product.warehouse_id.0
            ),
            None,
            None,
        )).unwrap();

        assert_eq!(expectation, result);
    }

    {
        let expectation = Some(warehouse_product.clone());
        let result = core.run(
            http_client.request_with_auth_header::<Option<WarehouseProduct>>(
                Method::Get,
                format!(
                    "{}/warehouses/{}/products/{}",
                    base_url, warehouse_product.warehouse_id.0, warehouse_product.product_id.0
                ),
                None,
                None,
            ),
        ).unwrap();

        assert_eq!(expectation, result);
    }

    core.run(http_client.request_with_auth_header::<Vec<Warehouse>>(
        Method::Post,
        format!("{}/warehouses/clear", base_url),
        None,
        None,
    )).unwrap();
}
