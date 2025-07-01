// standard library
use std::fmt;

pub fn format_expand_query<I>(expansions: I) -> Option<String>
where
    I: IntoIterator,
    I::Item: fmt::Display,
{
    let mut iter = expansions.into_iter().peekable();
    iter.peek()?;

    let mut query = String::new();
    for (i, expansion) in iter.enumerate() {
        if i == 0 {
            query.push_str(&format!("expand[]={expansion}"));
        } else {
            query.push_str(&format!("&expand[]={expansion}"));
        }
    }
    Some(query)
}
