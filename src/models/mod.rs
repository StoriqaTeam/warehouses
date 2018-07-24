pub mod warehouse;
pub use self::warehouse::*;

pub mod stock;
pub use self::stock::*;

pub mod role;
pub use self::role::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ValueContainer<T> {
    pub value: T,
}

impl<T> From<T> for ValueContainer<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}
