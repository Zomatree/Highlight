use regex::{Regex, RegexBuilder, escape};

use crate::Error;

pub fn create_highlight_regex(keywords: &[String]) -> Result<Regex, Error> {
    let keywords = keywords
        .iter()
        .map(|kw| escape(&kw))
        .collect::<Vec<_>>()
        .join("|");

    let patten = format!(r#"(?:^|[^\w])({keywords})(?:s|[^\w]|$)"#);

    RegexBuilder::new(&patten)
        .case_insensitive(true)
        .build()
        .map_err(|_| Error::InvalidKeyword)
}
