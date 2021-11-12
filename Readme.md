## Example

```rust
use fields_valid::FieldsValidate;

#[derive(FieldsValidate)]
struct User<'r> {
    // #[valid(len(5, 11), email, regex("^[\\d@\\.]+$"), "邮箱格式不正确")]
    #[valid(len(5, 17), "The length of the mailbox must be between 5-16")]
    #[valid(email, "E-mail format is incorrect")]
    email: String,
    #[valid(len(8, 17), "The minimum password length is 8 digits and the maximum is 16 digits")]
    password: String,
    #[valid(eq("#password"), "The two passwords are inconsistent")]
    password2: &'r str,
    #[valid(len(5, 12), "The length of the phone number is incorrect")]
    #[valid(regex("\\d+"), "The phone number must be a number")]
    phone: Option<String>,
    #[valid(range(5.8, 10.1), "Incorrect amount returned")]
    amount: u32,
}

#[inline]
fn fields_validate<T: FieldsValidate>(fields: &T) -> Result<(), &'static str> {
    fields.fields_validate()
}

fn main() {
    let user = User {
        email: "111@11a.com".to_string(),
        password: "12ndd232.23".to_string(),
        password2: "1225",
        phone: None, //Some("111111".to_string())
        amount: 7,
    };
    
    println!("{:?}", fields_validate(&user));
}
```
