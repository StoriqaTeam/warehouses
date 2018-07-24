use config::*;
use errors::*;
use models::*;
use repos;
use services::*;
pub mod routing;
use self::routing::*;
use types::*;

use failure;
use failure::ResultExt;
use futures::future;
use futures::prelude::*;
use hyper;
use hyper::{Delete, Get, Headers, Post, Put, Request};
use std::rc::Rc;
use stq_db::repo::*;
use stq_http::controller::{Controller, ControllerFuture};
use stq_http::request_util::{parse_body, serialize_future};
use stq_roles;
use stq_roles::{repo::RepoLogin, service::RoleService};
use stq_router::RouteParser;
use stq_types::*;

pub type ServiceFactoryFuture<T> = Box<Future<Item = Box<T>, Error = failure::Error>>;

#[derive(Clone)]
pub struct ServiceFactory {
    roles_service_factory: Rc<Fn(Option<UserId>) -> ServiceFactoryFuture<RoleService<UserRole>>>,
    warehouse_service_factory: Rc<Fn(Option<UserId>) -> ServiceFactoryFuture<WarehouseService>>,
}

pub struct ControllerImpl {
    route_parser: Rc<RouteParser<Route>>,
    service_factory: ServiceFactory,
}

impl ControllerImpl {
    pub fn new(db_pool: &DbPool, _config: &Config) -> Self {
        ControllerImpl {
            service_factory: ServiceFactory {
                roles_service_factory: Rc::new({
                    let db_pool = db_pool.clone();
                    move |caller_id| {
                        Box::new(
                            stq_roles::service::RoleServiceImpl::new(db_pool.clone(), caller_id)
                                .map(|service| Box::new(service) as Box<RoleService<UserRole>>),
                        )
                            as Box<
                                Future<Item = Box<RoleService<UserRole>>, Error = failure::Error>,
                            >
                    }
                })
                    as Rc<
                        Fn(
                            Option<UserId>
                        ) -> Box<
                            Future<Item = Box<RoleService<UserRole>>, Error = failure::Error>,
                        >,
                    >,
                warehouse_service_factory: Rc::new({
                    let db_pool = db_pool.clone();
                    move |caller_id: Option<UserId>| {
                        // Fetch roles for this user
                        Box::new(
                            match caller_id {
                                Some(user_id) => Box::new(
                                    db_pool
                                        .run(move |conn| {
                                            repos::roles::make_su_repo().select(
                                                conn,
                                                RoleFilter {
                                                    user_id: Some(user_id),
                                                    ..Default::default()
                                                },
                                            )
                                        })
                                        .map_err(|e| {
                                            e.context("Failed to fetch user roles").into()
                                        }),
                                )
                                    as Box<Future<Item = Vec<RoleEntry>, Error = failure::Error>>,
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
                                                                        id: Some(
                                                                            warehouse_id.into(),
                                                                        ),
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
                                                    Box::new(repos::roles::make_repo(if let Some(
                                                        caller_id,
                                                    ) =
                                                        caller_id
                                                    {
                                                        RepoLogin::User {
                                                            caller_id,
                                                            caller_roles: warehouse_role_source
                                                                .clone(),
                                                        }
                                                    } else {
                                                        RepoLogin::Anonymous
                                                    }))
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
            },
            route_parser: Rc::new(routing::make_router()),
        }
    }
}

pub fn extract_user_id(headers: &Headers) -> Result<Option<UserId>, failure::Error> {
    if let Some(auth) = headers.get::<hyper::header::Authorization<String>>() {
        let string_id = auth.0.clone();

        Ok(string_id
            .parse()
            .map(|v| Some(UserId(v)))
            .map_err(failure::Error::from)
            .context(format!("Failed to parse user ID: {}", string_id))
            .context(Error::UserIdParse)?)
    } else {
        Ok(None)
    }
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
            future::result(extract_user_id(&headers))
                .map_err(|e| e.context("Failed to extract user ID").into())
                .map(move |caller_id| (caller_id, service_factory))
                .and_then(|(caller_id, service_factory)| {
                    let warehouse_service_factory =
                        service_factory.warehouse_service_factory.clone();
                    let roles_service_factory = service_factory.roles_service_factory.clone();

                    future::ok(())
                        .and_then(move |_| (warehouse_service_factory)(caller_id))
                        .and_then(move |warehouse_service| {
                            (roles_service_factory)(caller_id).map(move |s| (warehouse_service, s))
                        })
                        .map(move |(warehouse_service, roles_service)| {
                            (warehouse_service, roles_service, caller_id)
                        })
                })
                .and_then(move |(warehouse_service, roles_service, _caller_id)| {
                    match (&method, route) {
                        (Get, Some(Route::Warehouse { warehouse_id })) => {
                            return serialize_future({
                                debug!("Received request to get warehouse {:?}", warehouse_id);
                                warehouse_service.get_warehouse(warehouse_id)
                            })
                        }
                        (Get, Some(Route::WarehousesByStore { store_id })) => {
                            return serialize_future({
                                debug!("Received request to get warehouses for store {}", store_id);
                                warehouse_service.get_warehouses_for_store(store_id)
                            })
                        }
                        (Post, Some(Route::Warehouses)) => {
                            return serialize_future({
                                debug!("Received request to create warehouse");
                                parse_body::<WarehouseInput>(payload)
                                    .and_then(move |data| warehouse_service.create_warehouse(data))
                            })
                        }
                        (Put, Some(Route::Warehouse { warehouse_id })) => {
                            return serialize_future({
                                debug!("Received request to update warehouse {:?}", warehouse_id);
                                parse_body::<WarehouseUpdateData>(payload).and_then(move |data| {
                                    warehouse_service.update_warehouse(warehouse_id, data)
                                })
                            })
                        }
                        (Delete, Some(Route::Warehouse { warehouse_id })) => {
                            return serialize_future({
                                debug!("Received request to delete warehouse {:?}", warehouse_id);
                                warehouse_service.delete_warehouse(warehouse_id)
                            })
                        }
                        (Delete, Some(Route::Warehouses)) => {
                            return serialize_future({
                                debug!("Received request to delete all warehouses");
                                warehouse_service.delete_all_warehouses()
                            })
                        }
                        (Get, Some(Route::StocksInWarehouse { warehouse_id })) => {
                            return serialize_future({
                                debug!(
                                    "Received request to get stocks of warehouse {:?}",
                                    warehouse_id
                                );
                                warehouse_service.list_products_in_warehouse(warehouse_id)
                            })
                        }
                        (
                            Get,
                            Some(Route::StockInWarehouse {
                                warehouse_id,
                                product_id,
                            }),
                        ) => {
                            return serialize_future({
                                debug!(
                                    "Received request to get stocks of product {} in warehouse {}",
                                    product_id, warehouse_id
                                );
                                warehouse_service.get_product_in_warehouse(warehouse_id, product_id)
                            })
                        }
                        (
                            Put,
                            Some(Route::StockInWarehouse {
                                warehouse_id,
                                product_id,
                            }),
                        ) => {
                            return serialize_future({
                                parse_body::<StockSetPayload>(payload).and_then(move |data| {
                                    debug!(
                                    "Received request to update stocks of product {} in warehouse {} with the following data {:?}",
                                    product_id, warehouse_id, &data
                                );
                                    warehouse_service.set_product_in_warehouse(
                                        warehouse_id,
                                        product_id,
                                        data.quantity,
                                    )
                                })
                            })
                        }
                        (Get, Some(Route::StocksByProductId { product_id })) => {
                            return serialize_future({
                                debug!(
                                    "Received request to get stocks of product {} in all warehouses",
                                    product_id
                                );
                                warehouse_service.find_by_product_id(product_id)
                            });
                        }
                        (method, Some(Route::Roles(route))) => {
                            let c = stq_roles::routing::Controller {
                                service: roles_service.into(),
                            };
                            if let Some(out) = c.call(method, &route, payload) {
                                return out;
                            }
                        }
                        (_, _) => {}
                    };

                    // Fallback
                    Box::new(future::err(
                        format_err!("Could not route request {} {}", method, uri.path())
                            .context(Error::InvalidRoute)
                            .into(),
                    ))
                }),
        )
    }
}
