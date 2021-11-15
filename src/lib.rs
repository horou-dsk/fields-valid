pub use fields_valid_derive::*;
pub use lazy_static::*;
pub use regex::*;
pub mod validates;

pub trait FieldsValidate {
    fn fields_validate(&self) -> std::result::Result<(), &'static str>;
}
