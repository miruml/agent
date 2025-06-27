// standard library
use std::fmt;


pub fn build_expand_query<I>(expansions: I) -> Option<String>
where
    I: IntoIterator,
    I::Item: fmt::Display,
{
    let mut iter = expansions.into_iter().peekable();
    if iter.peek().is_none() {
        None
    } else {
        let mut query = String::new();
        for expansion in iter {
            query.push_str(&format!("expand[]={}", expansion));
        }
        Some(query)
    }
}