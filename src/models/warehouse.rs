use geo::Point as GeoPoint;
use stq_api::{self, types::ValueContainer};
use stq_db::statement::*;
use stq_types::*;
use tokio_postgres;

const ID_COLUMN: &str = "id";
const STORE_ID_COLUMN: &str = "store_id";
const SLUG_COLUMN: &str = "slug";
const NAME_COLUMN: &str = "name";
const LOCATION_COLUMN: &str = "location";
const ADMINISTRATIVE_AREA_LEVEL_1_COLUMN: &str = "administrative_area_level_1";
const ADMINISTRATIVE_AREA_LEVEL_2_COLUMN: &str = "administrative_area_level_2";
const COUNTRY_COLUMN: &str = "country";
const COUNTRY_CODE_COLUMN: &str = "country_code";
const LOCALITY_COLUMN: &str = "locality";
const POLITICAL_COLUMN: &str = "political";
const POSTAL_CODE_COLUMN: &str = "postal_code";
const ROUTE_COLUMN: &str = "route";
const STREET_NUMBER_COLUMN: &str = "street_number";
const ADDRESS_COLUMN: &str = "address";
const PLACE_ID_COLUMN: &str = "place_id";

pub struct DbWarehouse(pub stq_api::warehouses::Warehouse);

impl From<tokio_postgres::rows::Row> for DbWarehouse {
    fn from(v: tokio_postgres::rows::Row) -> Self {
        DbWarehouse(stq_api::warehouses::Warehouse {
            id: WarehouseId(v.get(ID_COLUMN)),
            store_id: StoreId(v.get(STORE_ID_COLUMN)),
            slug: WarehouseSlug(v.get(SLUG_COLUMN)),
            name: v.get(NAME_COLUMN),
            location: v.get(LOCATION_COLUMN),
            administrative_area_level_1: v.get(ADMINISTRATIVE_AREA_LEVEL_1_COLUMN),
            administrative_area_level_2: v.get(ADMINISTRATIVE_AREA_LEVEL_2_COLUMN),
            country: v.get(COUNTRY_COLUMN),
            country_code: Some(Alpha3(v.get(COUNTRY_CODE_COLUMN))),
            locality: v.get(LOCALITY_COLUMN),
            political: v.get(POLITICAL_COLUMN),
            postal_code: v.get(POSTAL_CODE_COLUMN),
            route: v.get(ROUTE_COLUMN),
            street_number: v.get(STREET_NUMBER_COLUMN),
            address: v.get(ADDRESS_COLUMN),
            place_id: v.get(PLACE_ID_COLUMN),
        })
    }
}

impl Inserter for DbWarehouse {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        let mut b = InsertBuilder::new(table);

        b = b.with_arg(ID_COLUMN, self.0.id.0);
        b = b.with_arg(STORE_ID_COLUMN, self.0.store_id.0);
        b = b.with_arg(SLUG_COLUMN, self.0.slug.0);

        if let Some(name) = self.0.name {
            b = b.with_arg(NAME_COLUMN, name);
        }

        if let Some(location) = self.0.location {
            b = b.with_arg(LOCATION_COLUMN, location);
        }

        if let Some(administrative_area_level_1) = self.0.administrative_area_level_1 {
            b = b.with_arg(
                ADMINISTRATIVE_AREA_LEVEL_1_COLUMN,
                administrative_area_level_1,
            );
        }

        if let Some(administrative_area_level_2) = self.0.administrative_area_level_2 {
            b = b.with_arg(
                ADMINISTRATIVE_AREA_LEVEL_2_COLUMN,
                administrative_area_level_2,
            );
        }

        if let Some(country) = self.0.country {
            b = b.with_arg(COUNTRY_COLUMN, country.to_string());
        }

        if let Some(country_code) = self.0.country_code {
            b = b.with_arg(COUNTRY_CODE_COLUMN, country_code.to_string());
        }

        if let Some(locality) = self.0.locality {
            b = b.with_arg(LOCALITY_COLUMN, locality);
        }

        if let Some(political) = self.0.political {
            b = b.with_arg(POLITICAL_COLUMN, political);
        }

        if let Some(postal_code) = self.0.postal_code {
            b = b.with_arg(POSTAL_CODE_COLUMN, postal_code);
        }

        if let Some(route) = self.0.route {
            b = b.with_arg(ROUTE_COLUMN, route);
        }

        if let Some(street_number) = self.0.street_number {
            b = b.with_arg(STREET_NUMBER_COLUMN, street_number);
        }

        if let Some(address) = self.0.address {
            b = b.with_arg(ADDRESS_COLUMN, address);
        }

        if let Some(place_id) = self.0.place_id {
            b = b.with_arg(PLACE_ID_COLUMN, place_id);
        }

        b
    }
}

#[derive(Clone, Debug, Default)]
pub struct WarehouseFilter {
    pub id: Option<ValueContainer<WarehouseId>>,
    pub slug: Option<ValueContainer<WarehouseSlug>>,
    pub store_id: Option<ValueContainer<StoreId>>,
    pub name: Option<ValueContainer<Option<String>>>,
    pub location: Option<ValueContainer<Option<GeoPoint<f64>>>>,
    pub administrative_area_level_1: Option<ValueContainer<Option<String>>>,
    pub administrative_area_level_2: Option<ValueContainer<Option<String>>>,
    pub country: Option<ValueContainer<Option<String>>>,
    pub country_code: Option<ValueContainer<Option<Alpha3>>>,
    pub locality: Option<ValueContainer<Option<String>>>,
    pub political: Option<ValueContainer<Option<String>>>,
    pub postal_code: Option<ValueContainer<Option<String>>>,
    pub route: Option<ValueContainer<Option<String>>>,
    pub street_number: Option<ValueContainer<Option<String>>>,
    pub address: Option<ValueContainer<Option<String>>>,
    pub place_id: Option<ValueContainer<Option<String>>>,
}

impl Filter for WarehouseFilter {
    fn into_filtered_operation_builder(self, table: &'static str) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(table);

        if let Some(id) = self.id {
            b = b.with_filter(ID_COLUMN, id.value.0);
        }

        if let Some(store_id) = self.store_id {
            b = b.with_filter(STORE_ID_COLUMN, store_id.value.0)
        }

        if let Some(slug) = self.slug {
            b = b.with_filter(SLUG_COLUMN, slug.value.0);
        }

        if let Some(name) = self.name {
            b = b.with_filter(NAME_COLUMN, name.value);
        }

        if let Some(location) = self.location {
            b = b.with_filter(LOCATION_COLUMN, location.value);
        }

        if let Some(administrative_area_level_1) = self.administrative_area_level_1 {
            b = b.with_filter(
                ADMINISTRATIVE_AREA_LEVEL_1_COLUMN,
                administrative_area_level_1.value,
            );
        }

        if let Some(administrative_area_level_2) = self.administrative_area_level_2 {
            b = b.with_filter(
                ADMINISTRATIVE_AREA_LEVEL_2_COLUMN,
                administrative_area_level_2.value,
            );
        }

        if let Some(country) = self.country {
            b = b.with_filter(COUNTRY_COLUMN, country.value.map(|v| v.to_string()));
        }

        if let Some(country_code) = self.country_code {
            b = b.with_filter(
                COUNTRY_CODE_COLUMN,
                country_code.value.map(|v| v.to_string()),
            );
        }

        if let Some(locality) = self.locality {
            b = b.with_filter(LOCALITY_COLUMN, locality.value);
        }

        if let Some(political) = self.political {
            b = b.with_filter(POLITICAL_COLUMN, political.value);
        }

        if let Some(postal_code) = self.postal_code {
            b = b.with_filter(POSTAL_CODE_COLUMN, postal_code.value);
        }

        if let Some(route) = self.route {
            b = b.with_filter(ROUTE_COLUMN, route.value);
        }

        if let Some(street_number) = self.street_number {
            b = b.with_filter(STREET_NUMBER_COLUMN, street_number.value);
        }

        if let Some(address) = self.address {
            b = b.with_filter(ADDRESS_COLUMN, address.value);
        }

        if let Some(place_id) = self.place_id {
            b = b.with_filter(PLACE_ID_COLUMN, place_id.value);
        }

        b
    }
}

impl From<WarehouseIdentifier> for WarehouseFilter {
    fn from(v: WarehouseIdentifier) -> Self {
        use stq_types::WarehouseIdentifier::*;

        match v {
            Id(id) => Self {
                id: Some(id.into()),
                ..Default::default()
            },
            Slug(slug) => Self {
                slug: Some(slug.into()),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WarehouseUpdater {
    pub mask: WarehouseFilter,
    pub data: stq_api::warehouses::WarehouseUpdateData,
}

impl Updater for WarehouseUpdater {
    fn into_update_builder(self, table: &'static str) -> UpdateBuilder {
        let Self { mask, data } = self;

        let mut b = UpdateBuilder::from(mask.into_filtered_operation_builder(table));

        if let Some(slug) = data.slug {
            b = b.with_value(SLUG_COLUMN, slug.value.0);
        }

        if let Some(name) = data.name {
            b = b.with_value(NAME_COLUMN, name.value);
        }

        if let Some(location) = data.location {
            b = b.with_value(LOCATION_COLUMN, location.value);
        }

        if let Some(administrative_area_level_1) = data.administrative_area_level_1 {
            b = b.with_value(
                ADMINISTRATIVE_AREA_LEVEL_1_COLUMN,
                administrative_area_level_1.value,
            );
        }

        if let Some(administrative_area_level_2) = data.administrative_area_level_2 {
            b = b.with_value(
                ADMINISTRATIVE_AREA_LEVEL_2_COLUMN,
                administrative_area_level_2.value,
            );
        }

        if let Some(country) = data.country {
            b = b.with_value(COUNTRY_COLUMN, country.value);
        }

        if let Some(country_code) = data.country_code {
            b = b.with_value(COUNTRY_CODE_COLUMN, country_code.value.map(|v| v.0));
        }

        if let Some(locality) = data.locality {
            b = b.with_value(LOCALITY_COLUMN, locality.value);
        }

        if let Some(political) = data.political {
            b = b.with_value(POLITICAL_COLUMN, political.value);
        }

        if let Some(postal_code) = data.postal_code {
            b = b.with_value(POSTAL_CODE_COLUMN, postal_code.value);
        }

        if let Some(route) = data.route {
            b = b.with_value(ROUTE_COLUMN, route.value);
        }

        if let Some(street_number) = data.street_number {
            b = b.with_value(STREET_NUMBER_COLUMN, street_number.value);
        }

        if let Some(address) = data.address {
            b = b.with_value(ADDRESS_COLUMN, address.value);
        }

        if let Some(place_id) = data.place_id {
            b = b.with_value(PLACE_ID_COLUMN, place_id.value);
        }

        b
    }
}
