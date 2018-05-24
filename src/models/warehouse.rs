use super::ValueContainer;
use failure;
use geo::Point as GeoPoint;
use iso_country::Country;
use std::fmt;
use std::str::FromStr;
use stq_db::repo::*;
use stq_db::statement::*;
use tokio_postgres;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct WarehouseId(pub i32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i32);

const ID_COLUMN: &'static str = "id";
const NAME_COLUMN: &'static str = "name";
const LOCATION_COLUMN: &'static str = "location";
const ADMINISTRATIVE_AREA_LEVEL_1_COLUMN: &'static str = "administrative_area_level_1";
const ADMINISTRATIVE_AREA_LEVEL_2_COLUMN: &'static str = "administrative_area_level_2";
const COUNTRY_COLUMN: &'static str = "country";
const LOCALITY_COLUMN: &'static str = "locality";
const POLITICAL_COLUMN: &'static str = "political";
const POSTAL_CODE_COLUMN: &'static str = "postal_code";
const ROUTE_COLUMN: &'static str = "route";
const STREET_NUMBER_COLUMN: &'static str = "street_number";
const ADDRESS_COLUMN: &'static str = "address";
const PLACE_ID_COLUMN: &'static str = "place_id";
const KIND_COLUMN: &'static str = "kind";

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WarehouseKind {
    #[serde(rename = "distribution_center")]
    DistributionCenter,
    #[serde(rename = "store")]
    Store,
}

impl fmt::Display for WarehouseKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::WarehouseKind::*;

        write!(
            f,
            "{}",
            match *self {
                DistributionCenter => "distribution_center",
                Store => "store",
            }
        )
    }
}

impl FromStr for WarehouseKind {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "distribution_center" => Ok(WarehouseKind::DistributionCenter),
            "store" => Ok(WarehouseKind::Store),
            _ => Err(format_err!("Unknown warehouse kind: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: WarehouseId,
    pub name: Option<String>,
    pub location: Option<GeoPoint<f64>>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<Country>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
    pub kind: WarehouseKind, // Immutable
}

impl From<tokio_postgres::rows::Row> for Warehouse {
    fn from(v: tokio_postgres::rows::Row) -> Self {
        Self {
            id: WarehouseId(v.get::<i32, _>(ID_COLUMN)),
            name: v.get(NAME_COLUMN),
            location: v.get(LOCATION_COLUMN),
            administrative_area_level_1: v.get(ADMINISTRATIVE_AREA_LEVEL_1_COLUMN),
            administrative_area_level_2: v.get(ADMINISTRATIVE_AREA_LEVEL_2_COLUMN),
            country: v.get::<Option<String>, _>(COUNTRY_COLUMN)
                .map(|v| v.parse().unwrap()),
            locality: v.get(LOCALITY_COLUMN),
            political: v.get(POLITICAL_COLUMN),
            postal_code: v.get(POSTAL_CODE_COLUMN),
            route: v.get(ROUTE_COLUMN),
            street_number: v.get(STREET_NUMBER_COLUMN),
            address: v.get(ADDRESS_COLUMN),
            place_id: v.get(PLACE_ID_COLUMN),
            kind: v.get::<String, _>(KIND_COLUMN).parse().unwrap(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarehouseInserter {
    pub name: Option<String>,
    pub location: Option<GeoPoint<f64>>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: Option<Country>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub place_id: Option<String>,
    pub kind: WarehouseKind,
}

impl WarehouseInserter {
    pub fn new(kind: WarehouseKind) -> Self {
        Self {
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
            kind,
        }
    }
}

impl Inserter for WarehouseInserter {
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        let mut b = InsertBuilder::new(table);

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

        b = b.with_arg(KIND_COLUMN, self.kind.to_string());

        b
    }
}

pub type WarehouseMeta = WarehouseInserter;

impl From<Warehouse> for (WarehouseId, WarehouseMeta) {
    fn from(v: Warehouse) -> Self {
        (
            v.id,
            WarehouseMeta {
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
                kind: v.kind,
            },
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct WarehouseFilter {
    pub id: Option<ValueContainer<WarehouseId>>,
    pub name: Option<ValueContainer<Option<String>>>,
    pub location: Option<ValueContainer<Option<GeoPoint<f64>>>>,
    pub administrative_area_level_1: Option<ValueContainer<Option<String>>>,
    pub administrative_area_level_2: Option<ValueContainer<Option<String>>>,
    pub country: Option<ValueContainer<Option<Country>>>,
    pub locality: Option<ValueContainer<Option<String>>>,
    pub political: Option<ValueContainer<Option<String>>>,
    pub postal_code: Option<ValueContainer<Option<String>>>,
    pub route: Option<ValueContainer<Option<String>>>,
    pub street_number: Option<ValueContainer<Option<String>>>,
    pub address: Option<ValueContainer<Option<String>>>,
    pub place_id: Option<ValueContainer<Option<String>>>,
    pub kind: Option<ValueContainer<WarehouseKind>>,
}

impl Filter for WarehouseFilter {
    fn into_filtered_operation_builder(
        self,
        op: FilteredOperation,
        table: &'static str,
    ) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(op, table);

        if let Some(id) = self.id {
            b = b.with_arg(ID_COLUMN, id.value.0);
        }

        if let Some(name) = self.name {
            b = b.with_arg(NAME_COLUMN, name.value);
        }

        if let Some(location) = self.location {
            b = b.with_arg(LOCATION_COLUMN, location.value);
        }

        if let Some(administrative_area_level_1) = self.administrative_area_level_1 {
            b = b.with_arg(
                ADMINISTRATIVE_AREA_LEVEL_1_COLUMN,
                administrative_area_level_1.value,
            );
        }

        if let Some(administrative_area_level_2) = self.administrative_area_level_2 {
            b = b.with_arg(
                ADMINISTRATIVE_AREA_LEVEL_2_COLUMN,
                administrative_area_level_2.value,
            );
        }

        if let Some(country) = self.country {
            b = b.with_arg(COUNTRY_COLUMN, country.value.map(|v| v.to_string()));
        }

        if let Some(locality) = self.locality {
            b = b.with_arg(LOCALITY_COLUMN, locality.value);
        }

        if let Some(political) = self.political {
            b = b.with_arg(POLITICAL_COLUMN, political.value);
        }

        if let Some(postal_code) = self.postal_code {
            b = b.with_arg(POSTAL_CODE_COLUMN, postal_code.value);
        }

        if let Some(route) = self.route {
            b = b.with_arg(ROUTE_COLUMN, route.value);
        }

        if let Some(street_number) = self.street_number {
            b = b.with_arg(STREET_NUMBER_COLUMN, street_number.value);
        }

        if let Some(address) = self.address {
            b = b.with_arg(ADDRESS_COLUMN, address.value);
        }

        if let Some(place_id) = self.place_id {
            b = b.with_arg(PLACE_ID_COLUMN, place_id.value);
        }

        if let Some(kind) = self.kind {
            b = b.with_arg(KIND_COLUMN, kind.value.to_string());
        }

        b
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WarehouseUpdateData {
    pub name: Option<ValueContainer<Option<String>>>,
    pub location: Option<ValueContainer<Option<GeoPoint<f64>>>>,
    pub administrative_area_level_1: Option<ValueContainer<Option<String>>>,
    pub administrative_area_level_2: Option<ValueContainer<Option<String>>>,
    pub country: Option<ValueContainer<Option<Country>>>,
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

        let mut b = UpdateBuilder::from(mask.into_filtered_operation_builder(
            FilteredOperation::Select,
            table,
        ));

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
            b = b.with_value(COUNTRY_COLUMN, country.value.map(|v| v.to_string()));
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
