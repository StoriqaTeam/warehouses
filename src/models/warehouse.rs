use super::ValueContainer;
use geo::Point as GeoPoint;
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
const LOCALITY_COLUMN: &str = "locality";
const POLITICAL_COLUMN: &str = "political";
const POSTAL_CODE_COLUMN: &str = "postal_code";
const ROUTE_COLUMN: &str = "route";
const STREET_NUMBER_COLUMN: &str = "street_number";
const ADDRESS_COLUMN: &str = "address";
const PLACE_ID_COLUMN: &str = "place_id";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: WarehouseId,
    pub store_id: StoreId,
    pub slug: WarehouseSlug,
    pub name: Option<String>,
    pub location: Option<GeoPoint<f64>>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
}

impl From<tokio_postgres::rows::Row> for Warehouse {
    fn from(v: tokio_postgres::rows::Row) -> Self {
        Self {
            id: WarehouseId(v.get(ID_COLUMN)),
            store_id: StoreId(v.get(STORE_ID_COLUMN)),
            slug: WarehouseSlug(v.get(SLUG_COLUMN)),
            name: v.get(NAME_COLUMN),
            location: v.get(LOCATION_COLUMN),
            administrative_area_level_1: v.get(ADMINISTRATIVE_AREA_LEVEL_1_COLUMN),
            administrative_area_level_2: v.get(ADMINISTRATIVE_AREA_LEVEL_2_COLUMN),
            country: v.get(COUNTRY_COLUMN),
            locality: v.get(LOCALITY_COLUMN),
            political: v.get(POLITICAL_COLUMN),
            postal_code: v.get(POSTAL_CODE_COLUMN),
            route: v.get(ROUTE_COLUMN),
            street_number: v.get(STREET_NUMBER_COLUMN),
            address: v.get(ADDRESS_COLUMN),
            place_id: v.get(PLACE_ID_COLUMN),
        }
    }
}

impl Inserter for Warehouse {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        let mut b = InsertBuilder::new(table);

        b = b.with_arg(ID_COLUMN, self.id.0);
        b = b.with_arg(STORE_ID_COLUMN, self.store_id.0);
        b = b.with_arg(SLUG_COLUMN, self.slug.0);

        if let Some(name) = self.name {
            b = b.with_arg(NAME_COLUMN, name);
        }

        if let Some(location) = self.location {
            b = b.with_arg(LOCATION_COLUMN, location);
        }

        if let Some(administrative_area_level_1) = self.administrative_area_level_1 {
            b = b.with_arg(
                ADMINISTRATIVE_AREA_LEVEL_1_COLUMN,
                administrative_area_level_1,
            );
        }

        if let Some(administrative_area_level_2) = self.administrative_area_level_2 {
            b = b.with_arg(
                ADMINISTRATIVE_AREA_LEVEL_2_COLUMN,
                administrative_area_level_2,
            );
        }

        if let Some(country) = self.country {
            b = b.with_arg(COUNTRY_COLUMN, country.to_string());
        }

        if let Some(locality) = self.locality {
            b = b.with_arg(LOCALITY_COLUMN, locality);
        }

        if let Some(political) = self.political {
            b = b.with_arg(POLITICAL_COLUMN, political);
        }

        if let Some(postal_code) = self.postal_code {
            b = b.with_arg(POSTAL_CODE_COLUMN, postal_code);
        }

        if let Some(route) = self.route {
            b = b.with_arg(ROUTE_COLUMN, route);
        }

        if let Some(street_number) = self.street_number {
            b = b.with_arg(STREET_NUMBER_COLUMN, street_number);
        }

        if let Some(address) = self.address {
            b = b.with_arg(ADDRESS_COLUMN, address);
        }

        if let Some(place_id) = self.place_id {
            b = b.with_arg(PLACE_ID_COLUMN, place_id);
        }

        b
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseInput {
    #[serde(default = "WarehouseId::new")]
    pub id: WarehouseId,
    pub store_id: StoreId,
    pub name: Option<String>,
    pub location: Option<GeoPoint<f64>>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
}

impl WarehouseInput {
    pub fn new(store_id: StoreId) -> Self {
        Self {
            store_id,
            id: WarehouseId::new(),
            name: Default::default(),
            location: Default::default(),
            administrative_area_level_1: Default::default(),
            administrative_area_level_2: Default::default(),
            country: Default::default(),
            locality: Default::default(),
            political: Default::default(),
            postal_code: Default::default(),
            route: Default::default(),
            street_number: Default::default(),
            address: Default::default(),
            place_id: Default::default(),
        }
    }

    pub fn split_slug(v: Warehouse) -> (WarehouseInput, WarehouseSlug) {
        (
            WarehouseInput {
                id: v.id,
                store_id: v.store_id,
                name: v.name,
                location: v.location,
                administrative_area_level_1: v.administrative_area_level_1,
                administrative_area_level_2: v.administrative_area_level_2,
                country: v.country,
                locality: v.locality,
                political: v.political,
                postal_code: v.postal_code,
                route: v.route,
                street_number: v.street_number,
                address: v.address,
                place_id: v.place_id,
            },
            v.slug,
        )
    }

    pub fn with_slug(self, slug: WarehouseSlug) -> Warehouse {
        Warehouse {
            id: self.id,
            store_id: self.store_id,
            slug,
            name: self.name,
            location: self.location,
            administrative_area_level_1: self.administrative_area_level_1,
            administrative_area_level_2: self.administrative_area_level_2,
            country: self.country,
            locality: self.locality,
            political: self.political,
            postal_code: self.postal_code,
            route: self.route,
            street_number: self.street_number,
            address: self.address,
            place_id: self.place_id,
        }
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WarehouseUpdateData {
    pub slug: Option<ValueContainer<WarehouseSlug>>,
    pub name: Option<ValueContainer<Option<String>>>,
    pub location: Option<ValueContainer<Option<GeoPoint<f64>>>>,
    pub administrative_area_level_1: Option<ValueContainer<Option<String>>>,
    pub administrative_area_level_2: Option<ValueContainer<Option<String>>>,
    pub country: Option<ValueContainer<Option<String>>>,
    pub locality: Option<ValueContainer<Option<String>>>,
    pub political: Option<ValueContainer<Option<String>>>,
    pub postal_code: Option<ValueContainer<Option<String>>>,
    pub route: Option<ValueContainer<Option<String>>>,
    pub street_number: Option<ValueContainer<Option<String>>>,
    pub address: Option<ValueContainer<Option<String>>>,
    pub place_id: Option<ValueContainer<Option<String>>>,
}

#[derive(Clone, Debug, Default)]
pub struct WarehouseUpdater {
    pub mask: WarehouseFilter,
    pub data: WarehouseUpdateData,
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
