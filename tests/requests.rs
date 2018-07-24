extern crate futures;
extern crate hyper;
#[macro_use]
extern crate maplit;
extern crate serde_json;
extern crate stq_http;
extern crate stq_types;
extern crate tokio_core;
extern crate warehouses_lib as lib;

pub mod common;

use hyper::Method;
use lib::controller::StockSetPayload;
use lib::models::*;
use stq_types::*;

#[test]
fn test_warehouses_service() {
    let common::Context {
        mut core,
        http_client,
        base_url,
    } = common::setup();

    let superuser_id = UserId(1);
    let superuser_auth_header = superuser_id.0.to_string();

    core.run(http_client.request_with_auth_header::<Vec<Warehouse>>(
        Method::Delete,
        format!("{}/warehouses", base_url),
        None,
        Some(superuser_auth_header.clone()),
    )).unwrap();

    let user_id = UserId(123114);

    core.run(http_client.request_with_auth_header::<Option<RoleEntry>>(
        Method::Delete,
        format!("{}/roles/by-user-id/{}", base_url, user_id.0),
        None,
        Some(superuser_auth_header.clone()),
    )).unwrap();

    let store_id = StoreId(423452345);

    let test_user = RoleEntry {
        id: RoleEntryId::new(),
        user_id,
        role: UserRole::StoreManager(store_id),
    };

    let test_user_auth_header = test_user.user_id.0.to_string();

    {
        let res = core.run(http_client.request_with_auth_header::<RoleEntry>(
            Method::Post,
            format!("{}/roles", base_url),
            Some(serde_json::to_string(&test_user).unwrap()),
            Some(superuser_auth_header.clone()),
        )).unwrap();

        assert_eq!(test_user, res);
    }

    let mut warehouse = {
        let id = WarehouseId::new();
        let input = WarehouseInput {
            id: id.clone(),
            name: Some("My warehouse".into()),
            ..WarehouseInput::new(store_id)
        };

        let res = core.run(http_client.request_with_auth_header::<Warehouse>(
            Method::Post,
            format!("{}/warehouses", base_url),
            Some(serde_json::to_string(&input).unwrap()),
            Some(test_user_auth_header.clone()),
        )).unwrap();

        let v = input.with_slug(res.slug.clone());

        assert_eq!(res, v);

        v
    };

    {
        let updater = WarehouseUpdateData {
            name: Some(Some("My warehouse".to_string()).into()),
            ..Default::default()
        };

        let res = core.run(http_client.request_with_auth_header::<Option<Warehouse>>(
            Method::Put,
            format!("{}/warehouses/by-id/{}", base_url, warehouse.id.0),
            Some(serde_json::to_string(&updater).unwrap()),
            Some(test_user_auth_header.clone()),
        )).unwrap();

        warehouse.name = Some("My warehouse".to_string());

        assert_eq!(Some(warehouse.clone()), res);
    }

    let mut warehouse_product = {
        let new_product_id = ProductId(2341241);
        let quantity = Quantity(4433);

        let update_data = StockSetPayload { quantity };

        let res = core.run(http_client.request_with_auth_header::<Stock>(
            Method::Put,
            format!(
                "{}/warehouses/by-id/{}/products/{}",
                base_url, warehouse.id.0, new_product_id.0
            ),
            Some(serde_json::to_string(&update_data).unwrap()),
            Some(test_user_auth_header.clone()),
        )).unwrap();

        let (id, _warehouse_id, _product_id, _meta) =
            <(StockId, WarehouseId, ProductId, StockMeta)>::from(res.clone());

        let expectation = Stock {
            id,
            warehouse_id: warehouse.id,
            product_id: new_product_id,
            quantity,
        };

        assert_eq!(expectation, res.clone());

        res
    };

    {
        let update_data = StockSetPayload {
            quantity: Quantity(7634),
        };

        let res = core.run(http_client.request_with_auth_header::<Stock>(
            Method::Put,
            format!(
                "{}/warehouses/by-id/{}/products/{}",
                base_url, warehouse.id.0, warehouse_product.product_id.0
            ),
            Some(serde_json::to_string(&update_data).unwrap()),
            Some(test_user_auth_header.clone()),
        )).unwrap();

        warehouse_product.quantity = update_data.quantity;

        let expectation = warehouse_product.clone();

        assert_eq!(expectation, res.clone());
    }

    {
        let (_id, _warehouse_id, product_id, meta) =
            <(StockId, WarehouseId, ProductId, StockMeta)>::from(warehouse_product.clone());

        let expectation = hashmap! {
            product_id => meta,
        };
        let result = core.run(http_client.request_with_auth_header::<StockMap>(
            Method::Get,
            format!(
                "{}/warehouses/by-id/{}/products",
                base_url, warehouse_product.warehouse_id.0
            ),
            None,
            Some(test_user_auth_header.clone()),
        )).unwrap();

        assert_eq!(expectation, result);
    }

    {
        let expectation = Some(warehouse_product.clone());
        let result = core.run(http_client.request_with_auth_header::<Option<Stock>>(
            Method::Get,
            format!(
                "{}/warehouses/by-id/{}/products/{}",
                base_url, warehouse_product.warehouse_id.0, warehouse_product.product_id.0
            ),
            None,
            Some(test_user_auth_header.clone()),
        )).unwrap();

        assert_eq!(expectation, result);
    }

    core.run(http_client.request_with_auth_header::<Vec<Warehouse>>(
        Method::Delete,
        format!("{}/warehouses", base_url),
        None,
        Some(superuser_auth_header.clone()),
    )).unwrap();
}
