table! {
    roles (id) {
        id -> Uuid,
        user_id -> Int4,
        name -> Varchar,
        data -> Jsonb,
    }
}

table! {
    stocks (id) {
        id -> Uuid,
        warehouse_id -> Nullable<Uuid>,
        product_id -> Int4,
        quantity -> Int4,
    }
}

table! {
    warehouses (id) {
        id -> Uuid,
        store_id -> Int4,
        slug -> Varchar,
        name -> Nullable<Varchar>,
        location -> Nullable<Point>,
        administrative_area_level_1 -> Nullable<Varchar>,
        administrative_area_level_2 -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        locality -> Nullable<Varchar>,
        political -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        route -> Nullable<Varchar>,
        street_number -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        place_id -> Nullable<Varchar>,
        country_code -> Nullable<Varchar>,
    }
}

joinable!(stocks -> warehouses (warehouse_id));

allow_tables_to_appear_in_same_query!(
    roles,
    stocks,
    warehouses,
);
