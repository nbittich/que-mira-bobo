use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use spargebra::{Query, Update};
use tui::widgets::TableState;
use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Url,
    Query,
    Submit,
    //  SavedPrefixes,
    //   SavedQueries,
    Output,
}

#[derive(Default)]
pub struct SparqlContext<'a> {
    pub url: TextArea<'a>,
    pub query: TextArea<'a>,
    pub output: Option<SparqlResponse>,
    pub mode: Option<Mode>,
    pub output_state: TableState,
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
    pub fn next_line_output(&mut self) {
        let len = if let Some(result) = &self.output {
            result.results.bindings.len()
        } else {
            0
        };
        let state = &mut self.output_state;
        let i = match state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    pub fn previous_line_output(&mut self) {
        let len = if let Some(result) = &self.output {
            result.results.bindings.len()
        } else {
            0
        };
        let state = &mut self.output_state;
        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SparqlResponse {
    pub head: Head,
    pub results: SparqlResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    pub link: Option<Vec<String>>,
    pub vars: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparqlResult {
    pub distinct: Option<bool>,
    pub bindings: Vec<BTreeMap<String, Binding>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Binding {
    pub datatype: Option<String>,
    #[serde(rename = "type")]
    pub rdf_type: String,
    pub value: String,
    #[serde(rename = "xml:lang")]
    pub lang: Option<String>,
}
