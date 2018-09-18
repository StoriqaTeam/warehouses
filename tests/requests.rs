extern crate futures;
#[macro_use]
extern crate maplit;
extern crate serde_json;
extern crate stq_api;
extern crate stq_http;
extern crate stq_roles;
extern crate stq_types;
extern crate tokio;
extern crate tokio_core;
extern crate warehouses_lib as lib;

pub mod common;

use futures::{future, prelude::*};
use lib::models::*;
use stq_api::{roles::*, rpc_client::RestApiClient, warehouses::*};
use stq_roles::models::RoleSearchTerms;
use stq_types::*;

#[test]
fn test_services() {
    let base_url = common::setup();

    tokio::run(future::ok(()).map(move |_| {
        let superuser_id = UserId(1);
        let su_rpc_client = RestApiClient::new(&base_url, Some(superuser_id));

        su_rpc_client.delete_all_warehouses().wait().unwrap();

        let user_id = UserId(123114);
        let store_id = StoreId(423452345);

        RolesClient::<UserRole>::remove_role(
            &su_rpc_client,
            RoleSearchTerms::Meta((user_id, None)),
        ).wait()
        .unwrap();
        let test_user = RoleEntry {
            id: RoleEntryId::new(),
            user_id,
            role: UserRole::StoreManager(store_id),
        };

        {
            let res = su_rpc_client.create_role(test_user.clone()).wait().unwrap();

            assert_eq!(test_user, res);
        }

        let rpc_client = RestApiClient::new(&base_url, Some(user_id));

        let mut warehouse = {
            let id = WarehouseId::new();
            let input = WarehouseInput {
                id: id.clone(),
                name: Some("My warehouse".into()),
                location: Some((37.62, 55.75).into()),
                ..WarehouseInput::new(store_id)
            };

            let res = rpc_client.create_warehouse(input.clone()).wait().unwrap();

            let v = input.with_slug(res.slug.clone());

            assert_eq!(res, v);

            v
        };

        {
            let updater = WarehouseUpdateData {
                name: Some(Some("My warehouse".to_string()).into()),
                ..Default::default()
            };

            let res = rpc_client
                .update_warehouse(warehouse.id.into(), updater)
                .wait()
                .unwrap();

            warehouse.name = Some("My warehouse".to_string());

            assert_eq!(Some(warehouse.clone()), res);
        }

        let mut warehouse_product = {
            let new_product_id = ProductId(2341241);
            let quantity = Quantity(4433);

            let res = rpc_client
                .set_product_in_warehouse(warehouse.id, new_product_id, quantity)
                .wait()
                .unwrap();

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
            let quantity = Quantity(7634);

            let res = rpc_client
                .set_product_in_warehouse(warehouse.id, warehouse_product.product_id, quantity)
                .wait()
                .unwrap();
            warehouse_product.quantity = quantity;

            let expectation = warehouse_product.clone();

            assert_eq!(expectation, res.clone());
        }

        {
            let (_id, _warehouse_id, product_id, meta) =
                <(StockId, WarehouseId, ProductId, StockMeta)>::from(warehouse_product.clone());

            let expectation = hashmap! {
                product_id => meta,
            };
            let result = rpc_client
                .list_products_in_warehouse(warehouse_product.warehouse_id)
                .wait()
                .unwrap();

            assert_eq!(expectation, result);
        }

        {
            let expectation = Some(warehouse_product.clone());
            let result = rpc_client
                .get_product_in_warehouse(
                    warehouse_product.warehouse_id,
                    warehouse_product.product_id,
                ).wait()
                .unwrap();

            assert_eq!(expectation, result);
        }

        su_rpc_client.delete_all_warehouses().wait().unwrap();
    }));
}
