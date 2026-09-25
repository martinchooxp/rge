use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use serde_json::Result;

use tui::widgets::{ Cell, Row, Table, };

pub mod r#type;
pub use r#type::Type;

pub mod data;
pub use data::{Data, SubnodeBegin, SubnodeMatch, Begin, Match};

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    begin: Begin,
    r#match: Vec<Match>,
    end: Data,
    after_context: usize,
    before_context: usize,
}

impl Node {
    pub fn line_number_at(&self, i: usize) -> Option<usize> {
        self.r#match.get(i).map(|m| m.line_number)
    }

    pub fn file_name(&self) -> String {
        self.begin.path.text.clone()
    }

    pub fn include_filter(&self, folder_filter: &[String]) -> bool {
        let non_empty: Vec<&String> = folder_filter.iter().filter(|f| !f.is_empty()).collect();
        if non_empty.is_empty() {
            return true;
        }
        non_empty.iter().any(|file| self.begin.path.text.contains(*file))
    }

    /// Returns true if the node matches the given search term by coincidence:
    /// either in the file name or in any of its matched lines.
    #[allow(dead_code)]
    pub fn matches_search(&self, term: &str) -> bool {
        let term = term.to_lowercase();
        if term.is_empty() {
            return true;
        }
        if self.begin.path.text.to_lowercase().contains(&term) {
            return true;
        }
        self.r#match.iter().any(|m| m.lines.text.to_lowercase().contains(&term))
    }

    pub fn new(data_raw: Vec<(&str, Type)>, after_context: usize, before_context: usize) -> Option<Self> {
        let mut begin: Option<Begin> = None;
        let mut r#match: Vec<Match> = vec![];
        let mut end: Option<Data> = None;
        for (d, t) in data_raw {
            match t {
                Type::begin => {
                    if let Ok(sub) = Self::parse_subnode_begin(d) {
                        begin = Some(sub.data);
                    }
                }
                Type::r#match => {
                    if let Ok(sub) = Self::parse_subnode_match(d) {
                        r#match.push(sub.data);
                    }
                }
                Type::context => {
                    if let Ok(sub) = Self::parse_subnode_match(d) {
                        r#match.push(sub.data);
                    }
                }
                Type::end => {
                    if let Ok(e) = Self::parse_data(d) {
                        end = Some(e);
                    }
                }
                _ => {}
            }
        }
        let begin = begin?;
        let end = end?;
        Some(Self {
            begin,
            r#match,
            end,
            after_context,
            before_context,
        })
    }

    fn parse_subnode_begin(d: &str) -> Result<SubnodeBegin> {
        let n: SubnodeBegin = serde_json::from_str(d)?;
        Ok(n)
    }

    fn parse_subnode_match(d: &str) -> Result<SubnodeMatch> {
        let n: SubnodeMatch = serde_json::from_str(d)?;
        Ok(n)
    }

    fn parse_data(d: &str) -> Result<Data> {
        let n: Data = serde_json::from_str(d)?;
        Ok(n)
    }

    pub fn len_matches_all(&self) -> usize {
        self.r#match.len()
    }

    pub fn detail(&self, offset_detail: usize) -> Table<'_> {
        let start = offset_detail.min(self.r#match.len());
        Table::new(
            self.r#match[start..].iter().map(|m| {
                Row::new(vec![
                    Cell::from(m.pretty_line_match()),
                ])
            })
        )
    }

    pub fn summary(&self) -> String {
        format!(" { }", self.begin.path.text)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        // write!(f, "Rg Explorer: {:?}\n{:?}", self.r#type, self.data)
        write!(f, "TODO!!!!")
    }
}


