use std::collections::BTreeMap;

use spargebra::{Query, Update};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Url,
    Query,
    SavedPrefixes,
    SavedQueries,
    Output,
}

#[derive(Debug, Default)]
pub struct SparqlContext {
    pub url: String,
    pub query: String,
    pub output: Option<String>,
    pub output_type: Option<String>,
    pub prefixes: BTreeMap<String, String>,
    pub saved_queries: BTreeMap<String, String>,
    pub mode: Option<Mode>,
    pub pos_cursor: (u16, u16),
}

impl SparqlContext {
    pub fn format_query(&mut self) -> Result<(), SparqlContextError> {
        let query_formatted = match Query::parse(&self.query, None) {
            Ok(q) => q.to_string(),
            Err(query_error) => match Update::parse(&self.query, None) {
                Ok(u) => u.to_string(),
                Err(update_error) => {
                    return Err(SparqlContextError::InvalidQuery(vec![
                        query_error.to_string(),
                        update_error.to_string(),
                    ]))
                }
            },
        };
        self.query = query_formatted;
        Ok(())
    }

    pub fn format_prefix(&self, prefix: &str) -> String {
        format!("<{}>", prefix)
    }
}

pub enum SparqlContextError {
    InvalidQuery(Vec<String>),
    //InvalidUrl,
    //InvalidPrefix,
    //UnknownSavedQuery,
}
