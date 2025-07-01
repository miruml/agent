// standard crates
use std::fmt;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum SearchOperator {
    Equals,
    Contains,
}

impl fmt::Display for SearchOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => write!(f, ":"),
            Self::Contains => write!(f, "~"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
        }
    }
}

pub fn format_search_clause<K, V, I>(key: K, op: SearchOperator, values: I, not: bool) -> String
where
    K: fmt::Display,
    V: fmt::Display,
    I: IntoIterator<Item = V>,
{
    if not {
        format!("-{}{}{}", key, op, join(values, ","))
    } else {
        format!("{}{}{}", key, op, join(values, ","))
    }
}

pub fn format_search_group<I>(clauses: I, op: LogicalOperator) -> Option<String>
where
    I: IntoIterator,
    I::Item: fmt::Display,
{
    let mut iter = clauses.into_iter().peekable();
    iter.peek()?;
    Some(join(iter, &format!(" {} ", op)))
}

pub fn join<I, T>(values: I, sep: &str) -> String
where
    I: IntoIterator<Item = T>,
    T: fmt::Display,
{
    let mut result = String::new();
    let mut iter = values.into_iter().peekable();
    while let Some(v) = iter.next() {
        write!(&mut result, "{}", v).unwrap();
        if iter.peek().is_some() {
            result.push_str(sep);
        }
    }
    result
}
