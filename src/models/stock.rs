use stq_api::{types::ValueContainer, warehouses::*};
use stq_db::statement::*;
use stq_types::*;
use tokio_postgres::rows::Row;

const ID_COLUMN: &str = "id";
const WAREHOUSE_ID_COLUMN: &str = "warehouse_id";
const PRODUCT_ID_COLUMN: &str = "product_id";
const QUANTITY_COLUMN: &str = "quantity";

pub struct DbStock(pub Stock);

impl From<Row> for DbStock {
    fn from(row: Row) -> Self {
        DbStock(Stock {
            id: StockId(row.get(ID_COLUMN)),
            warehouse_id: WarehouseId(row.get(WAREHOUSE_ID_COLUMN)),
            product_id: ProductId(row.get(PRODUCT_ID_COLUMN)),
            quantity: Quantity(row.get(QUANTITY_COLUMN)),
        })
    }
}

impl Inserter for DbStock {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        InsertBuilder::new(table)
            .with_arg(ID_COLUMN, self.0.id.0)
            .with_arg(PRODUCT_ID_COLUMN, self.0.product_id.0)
            .with_arg(QUANTITY_COLUMN, self.0.quantity.0)
            .with_arg(WAREHOUSE_ID_COLUMN, self.0.warehouse_id.0)
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
