use super::ServiceFuture;
use models::*;
use repos::*;
use types::DbPool;

use futures::prelude::*;
use std::sync::Arc;

pub trait WarehouseService {
    fn create_warehouse(&self, new_warehouse: WarehouseInserter) -> ServiceFuture<Warehouse>;
    fn get_warehouse(&self, warehouse_id: WarehouseId) -> ServiceFuture<Option<Warehouse>>;
    fn update_warehouse(
        &self,
        warehouse_id: WarehouseId,
        update_data: WarehouseUpdateData,
    ) -> ServiceFuture<Option<Warehouse>>;
    fn delete_warehouse(&self, warehouse_id: WarehouseId) -> ServiceFuture<Option<Warehouse>>;
    fn delete_all_warehouses(&self) -> ServiceFuture<Vec<Warehouse>>;

    fn add_product_to_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<WarehouseProduct>;
    fn update_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
        update_data: WarehouseProductUpdateData,
    ) -> ServiceFuture<Option<WarehouseProduct>>;
    fn get_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<Option<WarehouseProduct>>;
    fn delete_product_from_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<Option<WarehouseProduct>>;
    fn list_products_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
    ) -> ServiceFuture<WarehouseProductMap>;

    fn get_warehouse_product(
        &self,
        warehouse_product_id: WarehouseProductId,
    ) -> ServiceFuture<Option<WarehouseProduct>>;

    fn find_by_product_id(&self, product_id: ProductId) -> ServiceFuture<Vec<WarehouseProduct>>;
}

#[derive(Clone)]
pub struct RepoFactory {
    pub warehouse_repo_factory: Arc<Fn() -> Box<WarehouseRepo> + Send + Sync>,
    pub warehouse_products_repo_factory: Arc<Fn() -> Box<WarehouseProductsRepo> + Send + Sync>,
    pub warehouse_users_repo_factory: Arc<Fn() -> Box<WarehouseUsersRepo> + Send + Sync>,
}

pub struct WarehouseServiceImpl {
    pub repo_factory: RepoFactory,
    pub db_pool: DbPool,
}

impl WarehouseService for WarehouseServiceImpl {
    fn create_warehouse(&self, new_warehouse: WarehouseInserter) -> ServiceFuture<Warehouse> {
        let repo_factory = self.repo_factory.clone();
        Box::new(self.db_pool.run(move |conn| {
            (repo_factory.warehouse_repo_factory)()
                .insert_exactly_one(Box::new(conn), new_warehouse)
                .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
        }))
    }

    fn get_warehouse(&self, warehouse_id: WarehouseId) -> ServiceFuture<Option<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_repo_factory)()
                        .select(
                            Box::new(conn),
                            WarehouseFilter {
                                id: Some(warehouse_id.into()),
                                ..Default::default()
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|mut v| v.pop()),
        )
    }

    fn update_warehouse(
        &self,
        warehouse_id: WarehouseId,
        update_data: WarehouseUpdateData,
    ) -> ServiceFuture<Option<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_repo_factory)()
                        .update(
                            Box::new(conn),
                            WarehouseUpdater {
                                mask: WarehouseFilter {
                                    id: Some(warehouse_id.into()),
                                    ..Default::default()
                                },
                                data: update_data,
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|mut v| v.pop()),
        )
    }

    fn delete_warehouse(&self, warehouse_id: WarehouseId) -> ServiceFuture<Option<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_repo_factory)()
                        .delete(
                            Box::new(conn),
                            WarehouseFilter {
                                id: Some(warehouse_id.into()),
                                ..Default::default()
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|mut v| v.pop()),
        )
    }

    fn delete_all_warehouses(&self) -> ServiceFuture<Vec<Warehouse>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(self.db_pool.run(move |conn| {
            (repo_factory.warehouse_repo_factory)()
                .delete(
                    Box::new(conn),
                    WarehouseFilter {
                        ..Default::default()
                    },
                )
                .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
        }))
    }

    fn add_product_to_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<WarehouseProduct> {
        let repo_factory = self.repo_factory.clone();
        Box::new(self.db_pool.run({
            let repo_factory = repo_factory.clone();
            move |conn| {
                let repo = (repo_factory.warehouse_products_repo_factory)();

                repo.insert_exactly_one(
                    Box::new(conn),
                    WarehouseProductInserter {
                        warehouse_id,
                        product_id,
                        quantity: Quantity(0),
                    },
                ).map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                    .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
            }
        }))
    }

    fn update_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
        update_data: WarehouseProductUpdateData,
    ) -> ServiceFuture<Option<WarehouseProduct>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run({
                    let repo_factory = repo_factory.clone();
                    move |conn| {
                        let repo = (repo_factory.warehouse_products_repo_factory)();

                        repo.update(
                            Box::new(conn),
                            WarehouseProductUpdater {
                                mask: WarehouseProductFilter {
                                    warehouse_id: Some(warehouse_id.into()),
                                    product_id: Some(product_id.into()),
                                    ..Default::default()
                                },
                                data: update_data,
                            },
                        ).map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                            .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                    }
                })
                .map(|mut v| v.pop()),
        )
    }
    fn get_product_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<Option<WarehouseProduct>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)()
                        .select(
                            Box::new(conn),
                            WarehouseProductFilter {
                                warehouse_id: Some(warehouse_id.into()),
                                product_id: Some(product_id.into()),
                                ..Default::default()
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|mut warehouse_products| warehouse_products.pop()),
        )
    }
    fn delete_product_from_warehouse(
        &self,
        warehouse_id: WarehouseId,
        product_id: ProductId,
    ) -> ServiceFuture<Option<WarehouseProduct>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)()
                        .delete(
                            Box::new(conn),
                            WarehouseProductFilter {
                                warehouse_id: Some(warehouse_id.into()),
                                product_id: Some(product_id.into()),
                                ..Default::default()
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|mut warehouse_products| warehouse_products.pop()),
        )
    }
    fn list_products_in_warehouse(
        &self,
        warehouse_id: WarehouseId,
    ) -> ServiceFuture<WarehouseProductMap> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)()
                        .select(
                            Box::new(conn),
                            WarehouseProductFilter {
                                warehouse_id: Some(warehouse_id.into()),
                                ..Default::default()
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|v| {
                    v.into_iter()
                        .map(<(ProductId, WarehouseProductMeta)>::from)
                        .collect::<WarehouseProductMap>()
                }),
        )
    }
    fn find_by_product_id(&self, product_id: ProductId) -> ServiceFuture<Vec<WarehouseProduct>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(self.db_pool.run(move |conn| {
            (repo_factory.warehouse_products_repo_factory)()
                .select(
                    Box::new(conn),
                    WarehouseProductFilter {
                        product_id: Some(product_id.into()),
                        ..Default::default()
                    },
                )
                .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
        }))
    }
    fn get_warehouse_product(
        &self,
        warehouse_product_id: WarehouseProductId,
    ) -> ServiceFuture<Option<WarehouseProduct>> {
        let repo_factory = self.repo_factory.clone();
        Box::new(
            self.db_pool
                .run(move |conn| {
                    (repo_factory.warehouse_products_repo_factory)()
                        .select(
                            Box::new(conn),
                            WarehouseProductFilter {
                                id: Some(warehouse_product_id.into()),
                                ..Default::default()
                            },
                        )
                        .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                        .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
                })
                .map(|mut v| v.pop()),
        )
    }
}
