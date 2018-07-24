use super::ServiceFuture;
use models::*;
use repos::*;
use types::DbPool;

use futures::future;
use futures::prelude::*;
use std::rc::Rc;
use stq_roles;
use stq_types::*;

#[derive(Clone, Debug)]
pub enum RoleRemoveFilter {
    Id(RoleEntryId),
    Meta((UserId, Option<UserRole>)),
}

pub trait WarehouseService {
    fn create_warehouse(&self, new_warehouse: WarehouseInput) -> ServiceFuture<Warehouse>;
    fn get_warehouse(&self, warehouse_id: WarehouseIdentifier) -> ServiceFuture<Option<Warehouse>>;
    fn update_warehouse(
        &self,
        warehouse_id: WarehouseIdentifier,
        update_data: WarehouseUpdateData,
    ) -> ServiceFuture<Option<Warehouse>>;
    fn delete_warehouse(
        &self,
        warehouse_id: WarehouseIdentifier,
    ) -> ServiceFuture<Option<Warehouse>>;
    fn delete_all_warehouses(&self) -> ServiceFuture<Vec<Warehouse>>;
    fn get_warehouses_for_store(&self, store_id: StoreId) -> ServiceFuture<Vec<Warehouse>>;

    fn set_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
        quantity: Quantity,
    ) -> ServiceFuture<Stock>;
    fn get_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<Option<Stock>>;
    fn list_products_in_warehouse(&self, warehouse_id: WarehouseId) -> ServiceFuture<StockMap>;

    fn get_warehouse_product(&self, warehouse_product_id: StockId) -> ServiceFuture<Option<Stock>>;

    /// Find all products with id in all warehouses
    fn find_by_product_id(&self, product_id: ProductId) -> ServiceFuture<Vec<Stock>>;
}

#[derive(Clone)]
pub struct RepoFactory {
    pub warehouse_repo_factory: Rc<Fn() -> Box<WarehouseRepo>>,
    pub warehouse_slug_sequence_factory: Rc<Fn() -> Box<WarehouseSlugSequence>>,
    pub warehouse_products_repo_factory: Rc<Fn() -> Box<StocksRepo>>,
    pub warehouse_roles_repo_factory: Rc<Fn() -> Box<stq_roles::repo::RolesRepo<UserRole>>>,
}

pub struct WarehouseServiceImpl {
    pub repo_factory: RepoFactory,
    pub db_pool: DbPool,
}

impl WarehouseService for WarehouseServiceImpl {
    fn create_warehouse(&self, new_warehouse: WarehouseInput) -> ServiceFuture<Warehouse> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run({
                    let new_warehouse = new_warehouse.clone();
                    move |conn| {
                        future::ok(conn)
                            .and_then({
                                let f = repo_factory.warehouse_slug_sequence_factory.clone();
                                move |conn| (f)().next_val(conn)
                            })
                            .and_then({
                                let f = repo_factory.warehouse_repo_factory.clone();
                                move |(slug, conn)| {
                                    (f)().insert_exactly_one(
                                        conn,
                                        new_warehouse.with_slug(WarehouseSlug(slug.to_string())),
                                    )
                                }
                            })
                    }
                })
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to create warehouse with data: {:?}",
                        &new_warehouse
                    )).into()
                }),
        )
    }

    fn get_warehouse(&self, warehouse_id: WarehouseIdentifier) -> ServiceFuture<Option<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_repo_factory)().select(conn, warehouse_id.into())
                })
                .map(|mut v| v.pop()),
        )
    }

    fn get_warehouses_for_store(&self, store_id: StoreId) -> ServiceFuture<Vec<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_repo_factory)().select(
                        conn,
                        WarehouseFilter {
                            store_id: Some(store_id.into()),
                            ..Default::default()
                        },
                    )
                })
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to get warehouses for store: {}",
                        store_id.0
                    )).into()
                }),
        )
    }

    fn update_warehouse(
        &self,
        warehouse_id: WarehouseIdentifier,
        update_data: WarehouseUpdateData,
    ) -> ServiceFuture<Option<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run({
                    let update_data = update_data.clone();
                    let warehouse_id = warehouse_id.clone();
                    move |conn| {
                        (repo_factory.warehouse_repo_factory)().update(
                            conn,
                            WarehouseUpdater {
                                mask: warehouse_id.into(),
                                data: update_data,
                            },
                        )
                    }
                })
                .map(|mut v| v.pop())
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to update warehouse {:?} with data {:?}",
                        warehouse_id, &update_data
                    )).into()
                }),
        )
    }

    fn delete_warehouse(
        &self,
        warehouse_id: WarehouseIdentifier,
    ) -> ServiceFuture<Option<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run({
                    let warehouse_id = warehouse_id.clone();
                    move |conn| {
                        (repo_factory.warehouse_repo_factory)().delete(conn, warehouse_id.into())
                    }
                })
                .map(|mut v| v.pop())
                .map_err(move |e| {
                    e.context(format!("Failed to delete warehouse {:?}", warehouse_id))
                        .into()
                }),
        )
    }

    fn delete_all_warehouses(&self) -> ServiceFuture<Vec<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_repo_factory)().delete(
                        conn,
                        WarehouseFilter {
                            ..Default::default()
                        },
                    )
                })
                .map_err(|e| e.context("Failed to delete all warehouses").into()),
        )
    }

    fn set_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
        quantity: Quantity,
    ) -> ServiceFuture<Stock> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run({
                    let repo_factory = repo_factory.clone();
                    move |conn| {
                        let repo = (repo_factory.warehouse_products_repo_factory)();

                        repo.insert_exactly_one(
                            conn,
                            Stock {
                                id: StockId::new(),
                                warehouse_id,
                                product_id,
                                quantity,
                            },
                        )
                    }
                })
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to set product {} in warehouse {} to quantity {}",
                        product_id.0, warehouse_id.0, quantity.0
                    )).into()
                }),
        )
    }
    fn get_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<Option<Stock>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)().select(
                        conn,
                        StockFilter {
                            warehouse_id: Some(warehouse_id.into()),
                            product_id: Some(product_id.into()),
                            ..Default::default()
                        },
                    )
                })
                .map(|mut warehouse_products| warehouse_products.pop())
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to get product {} in warehouse {}",
                        product_id.0, warehouse_id.0
                    )).into()
                }),
        )
    }
    fn list_products_in_warehouse(&self, warehouse_id: WarehouseId) -> ServiceFuture<StockMap> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)().select(
                        conn,
                        StockFilter {
                            warehouse_id: Some(warehouse_id.into()),
                            ..Default::default()
                        },
                    )
                })
                .map(|v| {
                    v.into_iter()
                        .map(<(ProductId, StockMeta)>::from)
                        .collect::<StockMap>()
                })
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to list products in warehouse {}",
                        warehouse_id.0
                    )).into()
                }),
        )
    }
    fn find_by_product_id(&self, product_id: ProductId) -> ServiceFuture<Vec<Stock>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)().select(
                        conn,
                        StockFilter {
                            product_id: Some(product_id.into()),
                            ..Default::default()
                        },
                    )
                })
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to find warehouse products with product_id {}",
                        product_id.0
                    )).into()
                }),
        )
    }
    fn get_warehouse_product(&self, warehouse_product_id: StockId) -> ServiceFuture<Option<Stock>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)().select(
                        conn,
                        StockFilter {
                            id: Some(warehouse_product_id.into()),
                            ..Default::default()
                        },
                    )
                })
                .map(|mut v| v.pop())
                .map_err(move |e| {
                    e.context(format!(
                        "Failed to get warehouse product {}",
                        warehouse_product_id.0
                    )).into()
                }),
        )
    }
}
