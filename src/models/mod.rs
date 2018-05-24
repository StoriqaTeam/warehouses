pub mod warehouse;
pub use self::warehouse::*;

pub mod warehouse_product;
pub use self::warehouse_product::*;

pub mod warehouse_user;
pub use self::warehouse_user::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ValueContainer<T> {
    pub value: T,
}

impl<T> From<T> for ValueContainer<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}
