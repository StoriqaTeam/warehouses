CREATE TABLE warehouse_users (
    id SERIAL PRIMARY KEY,
    warehouse_id INTEGER REFERENCES warehouses (id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL,
    role VARCHAR NOT NULL,

    CONSTRAINT warehouse_user UNIQUE (warehouse_id, user_id)
);
