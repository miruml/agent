// external crates
use std::fmt;

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

pub struct SearchClause {
    pub key: String,
    pub op: SearchOperator,
    pub values: Vec<String>,
}

impl fmt::Display for SearchClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.key, self.op, self.values.join(","))
    }
}

pub struct SearchGroup {
    pub clauses: Vec<SearchClause>,
    pub op: LogicalOperator,
}

impl fmt::Display for SearchGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clause_strings: Vec<String> = self.clauses.iter().map(|c| c.to_string()).collect();
        write!(f, "{}", clause_strings.join(&format!(" {} ", self.op)))
    }
}
