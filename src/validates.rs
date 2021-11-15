lazy_static::lazy_static! {
    static ref EMAIL_REGEX: ::regex::Regex = ::regex::Regex::new("^[\\da-zA-Z_-]+@([\\da-zA-Z_-]+\\.[\\da-zA-Z_-]+)+$").unwrap();
}

#[inline]
pub fn email(v: &str) -> bool {
    EMAIL_REGEX.is_match(v)
}

#[inline]
pub fn match_regex(rx: &str, v: &str) -> bool {
    ::regex::Regex::new(rx).unwrap().is_match(v)
}
