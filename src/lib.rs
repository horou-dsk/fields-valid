pub use fields_valid_derive::*;
pub mod validates;

pub trait FieldsValidate {
    fn fields_validate(&self) -> std::result::Result<(), &'static str>;
}
