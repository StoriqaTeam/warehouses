CREATE TABLE stocks (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    warehouse_id UUID REFERENCES warehouses (id) ON DELETE CASCADE,
    product_id   INTEGER NOT NULL,
    quantity     INTEGER NOT NULL,

    CONSTRAINT stock UNIQUE (warehouse_id, product_id)
);
