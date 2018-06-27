CREATE SEQUENCE warehouse_slug_seq;
CREATE TABLE warehouses (
    id                          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    store_id                    INTEGER NOT NULL,
    slug                        VARCHAR UNIQUE NOT NULL DEFAULT nextval('warehouse_slug_seq')::text,
    name                        VARCHAR,
    location                    POINT,
    administrative_area_level_1 VARCHAR,
    administrative_area_level_2 VARCHAR,
    country                     VARCHAR,
    locality                    VARCHAR,
    political                   VARCHAR,
    postal_code                 VARCHAR,
    route                       VARCHAR,
    street_number               VARCHAR,
    address                     VARCHAR,
    place_id                    VARCHAR
);
