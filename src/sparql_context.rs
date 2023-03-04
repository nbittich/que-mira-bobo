use std::collections::BTreeMap;

use spargebra::{Query, Update};
use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Url,
    Query,
    SavedPrefixes,
    SavedQueries,
    Output,
}

#[derive(Default)]
pub struct SparqlContext<'a> {
    pub url: TextArea<'a>,
    pub query: TextArea<'a>,
    pub output: Option<String>,
    pub output_type: Option<String>,
    pub prefixes: BTreeMap<String, String>,
    pub saved_queries: BTreeMap<String, String>,
    pub mode: Option<Mode>,
    pub pos_cursor: (u16, u16),
}

impl SparqlContext<'_> {
    pub fn format_query(&mut self) -> Result<(), SparqlContextError> {
        let query = &self.query.lines().join("\n");
        let query_formatted = match Query::parse(query, None) {
            Ok(q) => q.to_string(),
            Err(query_error) => match Update::parse(query, None) {
                Ok(u) => u.to_string(),
                Err(update_error) => {
                    return Err(SparqlContextError::InvalidQuery(vec![
                        query_error.to_string(),
                        update_error.to_string(),
                    ]))
                }
            },
        };
        self.query = TextArea::from(query_formatted.lines());
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
