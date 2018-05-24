CREATE TABLE warehouse_products (
    id SERIAL PRIMARY KEY,
    warehouse_id INTEGER REFERENCES warehouses (id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity >= 0),

    CONSTRAINT warehouse_product UNIQUE (warehouse_id, product_id)
);
