#[inline]
pub fn email(v: &str) -> bool {
    let rx = regex::Regex::new("^[\\da-zA-Z_-]+@([\\da-zA-Z_-]+\\.[\\da-zA-Z_-]+)+$").unwrap();
    rx.is_match(v)
}

#[inline]
pub fn match_regex(rx: &str, v: &str) -> bool {
    regex::Regex::new(rx).unwrap().is_match(v)
}
