use config::*;
use errors::*;
use models::*;
use sentry_integration::log_and_capture_error;
use services::*;
use types::*;

use failure::{self, ResultExt};
use futures::{future, prelude::*};
use hyper::{self, Delete, Get, Headers, Post, Put, Request};
use std::rc::Rc;
use stq_api::warehouses::*;
use stq_http::{
    controller::{Controller, ControllerFuture},
    errors::ErrorMessageWrapper,
    request_util::{parse_body, serialize_future},
};
use stq_roles::{
    self,
    models::RepoLogin,
    service::{get_login_data, RoleService},
};
use stq_types::*;

pub const SUPERADMIN_USER: UserId = UserId(1);

#[derive(Clone)]
pub struct ServiceFactory {
    role: Rc<Fn(UserLogin) -> Box<RoleService<UserRole>>>,
    warehouse: Rc<Fn(UserLogin) -> Box<WarehouseService>>,
}

pub struct ControllerImpl {
    db_pool: DbPool,
    service_factory: ServiceFactory,
}

impl ControllerImpl {
    pub fn new(db_pool: DbPool, _config: &Config) -> Self {
        ControllerImpl {
            service_factory: {
                let roles_service_factory: Rc<
                    Fn(RepoLogin<UserRole>) -> Box<RoleService<UserRole>>,
                > = Rc::new({
                    let db_pool = db_pool.clone();
                    move |caller_login| {
                        Box::new(stq_roles::service::RoleServiceImpl::new(
                            db_pool.clone(),
                            caller_login,
                        ))
                    }
                });
                ServiceFactory {
                    role: roles_service_factory.clone(),
                    warehouse: Rc::new({
                        let db_pool = db_pool.clone();
                        move |login| {
                            Box::new(WarehouseServiceImpl::new(&db_pool, &login))
                                as Box<WarehouseService>
                        }
                    }),
                }
            },
            db_pool,
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

impl Controller for ControllerImpl {
    fn call(&self, request: Request) -> ControllerFuture {
        let (method, uri, _, headers, payload) = request.deconstruct();

        let service_factory = self.service_factory.clone();

        let route = Route::from_path(uri.path());

        Box::new(
            future::result(extract_user_id(&headers))
                .map_err(|e| e.context("Failed to extract user ID").into())
                .and_then({
                    let db_pool = self.db_pool.clone();
                    move |caller_id| get_login_data(&db_pool, caller_id)
                })
                .and_then(move |login_data| {
                    let warehouse_service = (service_factory.warehouse)(login_data.clone());
                    let roles_service = (service_factory.role)(login_data.clone());
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
                }).map_err(|err| {
                    let wrapper = ErrorMessageWrapper::<Error>::from(&err);
                    if wrapper.inner.code == 500 {
                        log_and_capture_error(&err);
                    }
                    err
                }),
        )
    }
}
