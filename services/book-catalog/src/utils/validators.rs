use std::sync::LazyLock;

use regex::Regex;

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

pub fn is_valid_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
}