use failure;
use failure::Fail;
use futures::future;
use futures::prelude::*;
use hyper;
use hyper::{Delete, Get, Headers, Post, Put, Request};
use std::rc::Rc;
use std::str::FromStr;

use stq_db::repo::*;
use stq_http::controller::{Controller, ControllerFuture};
use stq_http::request_util::{parse_body, serialize_future};
use stq_router::RouteParser;

use config::*;
use errors::*;
use models::*;
use repos;
use services::*;
pub mod routing;
use self::routing::*;
use types::*;

pub struct ControllerImpl {
    route_parser: Rc<RouteParser<Route>>,
    service_factory:
        Rc<Fn(Option<UserId>) -> Box<Future<Item = Box<WarehouseService>, Error = failure::Error>>>,
}

impl ControllerImpl {
    pub fn new(db_pool: DbPool, _config: Config) -> Self {
        ControllerImpl {
            service_factory: Rc::new({
                let db_pool = db_pool.clone();
                move |user_id: Option<UserId>| {
                    // Fetch roles for this user
                    Box::new(
                        match user_id {
                            Some(id) => Box::new(
                                db_pool
                                    .run(move |conn| {
                                        repos::roles::make_su_repo().select(
                                            conn,
                                            RoleFilter {
                                                user_id: Some(id.into()),
                                                ..Default::default()
                                            },
                                        )
                                    })
                                    .map_err(|e| e.context("Failed to fetch user roles").into()),
                            )
                                as Box<Future<Item = Vec<Role>, Error = failure::Error>>,
                            None => Box::new(future::ok(vec![])),
                        }.map({
                            let db_pool = db_pool.clone();
                            move |roles| {
                                Box::new(WarehouseServiceImpl {
                                    db_pool: db_pool.clone(),
                                    repo_factory: RepoFactory {
                                        warehouse_repo_factory: Rc::new({
                                            let warehouse_role_source = roles.clone();
                                            move || {
                                                Box::new(repos::warehouses::make_repo(
                                                    warehouse_role_source.clone(),
                                                ))
                                            }
                                        }),
                                        warehouse_slug_sequence_factory: Rc::new({
                                            || Box::new(repos::warehouses::make_slug_sequence())
                                        }),
                                        warehouse_products_repo_factory: Rc::new({
                                            let warehouse_role_source = roles.clone();
                                            let warehouse_source = Rc::new({
                                                let db_pool = db_pool.clone();
                                                move |warehouse_id: WarehouseId| {
                                                    Box::new(db_pool.run(move |conn| {
                                                        repos::warehouses::make_su_repo()
                                                            .select_exactly_one(
                                                                conn,
                                                                WarehouseFilter {
                                                                    id: Some(warehouse_id.into()),
                                                                    ..Default::default()
                                                                },
                                                            )
                                                    }))
                                                        as Box<
                                                            Future<
                                                                Item = Warehouse,
                                                                Error = failure::Error,
                                                            >,
                                                        >
                                                }
                                            });
                                            move || {
                                                Box::new(repos::stocks::make_repo(
                                                    warehouse_role_source.clone(),
                                                    warehouse_source.clone(),
                                                ))
                                            }
                                        }),
                                        warehouse_roles_repo_factory: Rc::new({
                                            let warehouse_role_source = roles.clone();
                                            move || {
                                                Box::new(repos::roles::make_repo(
                                                    warehouse_role_source.clone(),
                                                ))
                                            }
                                        }),
                                    },
                                }) as Box<WarehouseService>
                            }
                        }),
                    )
                        as Box<Future<Item = Box<WarehouseService>, Error = failure::Error>>
                }
            }),
            route_parser: Rc::new(routing::make_router()),
        }
    }
}

pub fn extract_user_id(
    headers: Headers,
) -> Box<Future<Item = Option<UserId>, Error = failure::Error>> {
    Box::new(future::result(match headers
        .get::<hyper::header::Authorization<String>>()
        .map(|auth| auth.0.clone())
    {
        None => Ok(None),
        Some(string_id) => i32::from_str(&string_id)
            .map(|v| Some(UserId(v)))
            .map_err(|e| {
                failure::Error::from(e)
                    .context(format!("Failed to parse user ID: {}", string_id))
                    .context(Error::UserIdParse)
                    .into()
            }),
    }))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockSetPayload {
    pub quantity: Quantity,
}

impl Controller for ControllerImpl {
    fn call(&self, request: Request) -> ControllerFuture {
        let (method, uri, _, headers, payload) = request.deconstruct();

        let service_factory = self.service_factory.clone();
        let route_parser = self.route_parser.clone();

        let route = route_parser.test(uri.path());

        Box::new(
            extract_user_id(headers)
                .map_err(|e| e.context("Failed to extract user ID").into())
                .map(move |user_id| (user_id, service_factory))
                .and_then(|(user_id, service_factory)| {
                    ((service_factory)(user_id).map(move |s| (s, user_id)))
                })
                .and_then(
                    move |(service, _requesting_user_id)| match (method, route) {
                        (Get, Some(Route::Warehouse { warehouse_id })) => serialize_future({
                            debug!("Received request to get warehouse {:?}", warehouse_id);
                            service.get_warehouse(warehouse_id)
                        }),
                        (Get, Some(Route::WarehousesByStore { store_id })) => serialize_future({
                            debug!("Received request to get warehouses for store {}", store_id);
                            service.get_warehouses_for_store(store_id)
                        }),
                        (Post, Some(Route::Warehouses)) => serialize_future({
                            debug!("Received request to create warehouse");
                            parse_body::<WarehouseInput>(payload)
                                .and_then(move |data| service.create_warehouse(data))
                        }),
                        (Put, Some(Route::Warehouse { warehouse_id })) => serialize_future({
                            debug!("Received request to update warehouse {:?}", warehouse_id);
                            parse_body::<WarehouseUpdateData>(payload)
                                .and_then(move |data| service.update_warehouse(warehouse_id, data))
                        }),
                        (Delete, Some(Route::Warehouse { warehouse_id })) => serialize_future({
                            debug!("Received request to delete warehouse {:?}", warehouse_id);
                            service.delete_warehouse(warehouse_id)
                        }),
                        (Delete, Some(Route::Warehouses)) => serialize_future({
                            debug!("Received request to delete all warehouses");
                            service.delete_all_warehouses()
                        }),
                        (Get, Some(Route::StocksInWarehouse { warehouse_id })) => {
                            serialize_future({
                                debug!(
                                    "Received request to get stocks of warehouse {:?}",
                                    warehouse_id
                                );
                                service.list_products_in_warehouse(warehouse_id)
                            })
                        }
                        (
                            Get,
                            Some(Route::StockInWarehouse {
                                warehouse_id,
                                product_id,
                            }),
                        ) => serialize_future({
                            debug!(
                                "Received request to get stocks of product {} in warehouse {}",
                                product_id, warehouse_id
                            );
                            service.get_product_in_warehouse(warehouse_id, product_id)
                        }),
                        (
                            Put,
                            Some(Route::StockInWarehouse {
                                warehouse_id,
                                product_id,
                            }),
                        ) => serialize_future({
                            parse_body::<StockSetPayload>(payload).and_then(move |data| {
                                debug!(
                        "Received request to update stocks of product {} in warehouse {} with the following data {:?}",
                        product_id, warehouse_id, &data
                    );
                                service.set_product_in_warehouse(
                                    warehouse_id,
                                    product_id,
                                    data.quantity,
                                )
                            })
                        }),
                        (Get, Some(Route::StocksByProductId { product_id })) => serialize_future({
                            debug!(
                                "Received request to get stocks of product {} in all warehouses",
                                product_id
                            );
                            service.find_by_product_id(product_id)
                        }),
                        (Get, Some(Route::RolesByUserId { user_id })) => {
                            serialize_future({ service.get_roles_for_user(user_id) })
                        }
                        (Post, Some(Route::Roles)) => serialize_future({
                            parse_body::<Role>(payload)
                                .and_then(move |data| service.create_role(data))
                        }),
                        (Delete, Some(Route::RolesByUserId { user_id })) => serialize_future({
                            parse_body::<Option<UserRole>>(payload).and_then(move |role| {
                                service.remove_role(RoleRemoveFilter::Meta((user_id, role)))
                            })
                        }),
                        (Delete, Some(Route::RoleById { role_id })) => {
                            serialize_future({ service.remove_role(RoleRemoveFilter::Id(role_id)) })
                        }
                        // Fallback
                        (other_method, _) => Box::new(future::err(
                            format_err!("Could not route request {} {}", other_method, uri.path())
                                .context(Error::InvalidRoute)
                                .into(),
                        )),
                    },
                ),
        )
    }
}
