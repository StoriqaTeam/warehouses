use super::warehouse::WarehouseId;
use super::ValueContainer;

use std::collections::HashMap;
use stq_db::statement::*;
use tokio_postgres::rows::Row;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Display, PartialEq, FromStr, Hash, Serialize, Deserialize)]
pub struct StockId(pub Uuid);
impl StockId {
    pub fn new() -> Self {
        StockId(Uuid::new_v4())
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, FromStr, Hash, Serialize, Deserialize)]
pub struct ProductId(pub i32);

#[derive(Clone, Copy, Debug, Display, PartialEq, FromStr, Hash, Serialize, Deserialize)]
pub struct Quantity(pub i32);

const ID_COLUMN: &'static str = "id";
const WAREHOUSE_ID_COLUMN: &'static str = "warehouse_id";
const PRODUCT_ID_COLUMN: &'static str = "product_id";
const QUANTITY_COLUMN: &'static str = "quantity";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    pub id: StockId,
    pub warehouse_id: WarehouseId,
    pub product_id: ProductId,
    pub quantity: Quantity,
}

impl From<Row> for Stock {
    fn from(row: Row) -> Self {
        Self {
            id: StockId(row.get(ID_COLUMN)),
            warehouse_id: WarehouseId(row.get(WAREHOUSE_ID_COLUMN)),
            product_id: ProductId(row.get(PRODUCT_ID_COLUMN)),
            quantity: Quantity(row.get(QUANTITY_COLUMN)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockMeta {
    pub quantity: Quantity,
}

impl From<Stock> for (ProductId, StockMeta) {
    fn from(v: Stock) -> (ProductId, StockMeta) {
        (
            v.product_id,
            StockMeta {
                quantity: v.quantity,
            },
        )
    }
}

impl From<Stock> for (StockId, WarehouseId, ProductId, StockMeta) {
    fn from(v: Stock) -> Self {
        (
            v.id,
            v.warehouse_id,
            v.product_id,
            StockMeta {
                quantity: v.quantity,
            },
        )
    }
}

pub type StockMap = HashMap<ProductId, StockMeta>;

impl Inserter for Stock {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        InsertBuilder::new(table)
            .with_arg(ID_COLUMN, self.id.0)
            .with_arg(PRODUCT_ID_COLUMN, self.product_id.0)
            .with_arg(QUANTITY_COLUMN, self.quantity.0)
            .with_arg(WAREHOUSE_ID_COLUMN, self.warehouse_id.0)
            .with_extra("ON CONFLICT (warehouse_id, product_id) DO UPDATE SET quantity = $3")
    }
}

#[derive(Clone, Debug, Default)]
pub struct StockFilter {
    pub id: Option<ValueContainer<StockId>>,
    pub warehouse_id: Option<ValueContainer<WarehouseId>>,
    pub product_id: Option<ValueContainer<ProductId>>,
    pub quantity: Option<ValueContainer<Quantity>>,
}

impl Filter for StockFilter {
    fn into_filtered_operation_builder(self, table: &'static str) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(table);

        if let Some(id) = self.id {
            b = b.with_filter(ID_COLUMN, id.value.0);
        }

        if let Some(warehouse_id) = self.warehouse_id {
            b = b.with_filter(WAREHOUSE_ID_COLUMN, warehouse_id.value.0);
        }

        if let Some(product_id) = self.product_id {
            b = b.with_filter(PRODUCT_ID_COLUMN, product_id.value.0);
        }

        if let Some(quantity) = self.quantity {
            b = b.with_filter(QUANTITY_COLUMN, quantity.value.0);
        }

        b
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StockUpdateData {
    pub quantity: Option<ValueContainer<Quantity>>,
}

#[derive(Clone, Debug, Default)]
pub struct StockUpdater {
    pub mask: StockFilter,
    pub data: StockUpdateData,
}

impl Updater for StockUpdater {
    fn into_update_builder(self, table: &'static str) -> UpdateBuilder {
        let mut b = UpdateBuilder::from(self.mask.into_filtered_operation_builder(table));

        if let Some(quantity) = self.data.quantity {
            b = b.with_value(QUANTITY_COLUMN, quantity.value.0);
        }

        b
    }
}
