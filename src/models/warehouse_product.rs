use super::ValueContainer;
use super::warehouse::WarehouseId;

use std::collections::HashMap;
use stq_db::repo::*;
use stq_db::statement::*;
use tokio_postgres::rows::Row;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WarehouseProductId(pub i32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ProductId(pub i32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Quantity(pub i32);

const ID_COLUMN: &'static str = "id";
const WAREHOUSE_ID_COLUMN: &'static str = "warehouse_id";
const PRODUCT_ID_COLUMN: &'static str = "product_id";
const QUANTITY_COLUMN: &'static str = "quantity";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseProduct {
    pub id: WarehouseProductId,
    pub warehouse_id: WarehouseId,
    pub product_id: ProductId,
    pub quantity: Quantity,
}

impl From<Row> for WarehouseProduct {
    fn from(row: Row) -> Self {
        Self {
            id: WarehouseProductId(row.get::<_, _>(ID_COLUMN)),
            warehouse_id: WarehouseId(row.get::<_, _>(WAREHOUSE_ID_COLUMN)),
            product_id: ProductId(row.get::<_, _>(PRODUCT_ID_COLUMN)),
            quantity: Quantity(row.get::<_, _>(QUANTITY_COLUMN)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseProductMeta {
    pub quantity: Quantity,
}

impl From<WarehouseProduct> for (ProductId, WarehouseProductMeta) {
    fn from(v: WarehouseProduct) -> (ProductId, WarehouseProductMeta) {
        (
            v.product_id,
            WarehouseProductMeta {
                quantity: v.quantity,
            },
        )
    }
}

impl From<WarehouseProduct>
    for (
        WarehouseProductId,
        WarehouseId,
        ProductId,
        WarehouseProductMeta,
    )
{
    fn from(v: WarehouseProduct) -> Self {
        (
            v.id,
            v.warehouse_id,
            v.product_id,
            WarehouseProductMeta {
                quantity: v.quantity,
            },
        )
    }
}

pub type WarehouseProductMap = HashMap<ProductId, WarehouseProductMeta>;

#[derive(Clone, Debug)]
pub struct WarehouseProductInserter {
    pub warehouse_id: WarehouseId,
    pub product_id: ProductId,
    pub quantity: Quantity,
}

impl Inserter for WarehouseProductInserter {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        InsertBuilder::new(table)
            .with_arg(WAREHOUSE_ID_COLUMN, self.warehouse_id.0)
            .with_arg(PRODUCT_ID_COLUMN, self.product_id.0)
            .with_arg(QUANTITY_COLUMN, self.quantity.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct WarehouseProductFilter {
    pub id: Option<ValueContainer<WarehouseProductId>>,
    pub warehouse_id: Option<ValueContainer<WarehouseId>>,
    pub product_id: Option<ValueContainer<ProductId>>,
    pub quantity: Option<ValueContainer<Quantity>>,
}

impl Filter for WarehouseProductFilter {
    fn into_filtered_operation_builder(
        self,
        op: FilteredOperation,
        table: &'static str,
    ) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(op, table);

        if let Some(id) = self.id {
            b = b.with_arg(ID_COLUMN, id.value.0);
        }

        if let Some(warehouse_id) = self.warehouse_id {
            b = b.with_arg(WAREHOUSE_ID_COLUMN, warehouse_id.value.0);
        }

        if let Some(product_id) = self.product_id {
            b = b.with_arg(PRODUCT_ID_COLUMN, product_id.value.0);
        }

        if let Some(quantity) = self.quantity {
            b = b.with_arg(QUANTITY_COLUMN, quantity.value.0);
        }

        b
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WarehouseProductUpdateData {
    pub quantity: Option<ValueContainer<Quantity>>,
}

#[derive(Clone, Debug, Default)]
pub struct WarehouseProductUpdater {
    pub mask: WarehouseProductFilter,
    pub data: WarehouseProductUpdateData,
}

impl Updater for WarehouseProductUpdater {
    fn into_update_builder(self, table: &'static str) -> UpdateBuilder {
        let mut b = UpdateBuilder::from(
            self.mask
                .into_filtered_operation_builder(FilteredOperation::Select, table),
        );

        if let Some(quantity) = self.data.quantity {
            b = b.with_value(QUANTITY_COLUMN, quantity.value.0);
        }

        b
    }
}
