use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{BufRead, BufReader};
use std::process::{ChildStdout, Command, Stdio};

use crossterm::event::KeyCode;

use crate::ui::FilterMode;
use crate::explorer::nodes::{Nodes, Node, Type};

pub mod nodes;

pub struct RipGrep {
    pub search_term: String,
    pub search_term_buffer: String,
    after_context: usize,
    before_context: usize,
    folder: String,
    reader: Option<BufReader<ChildStdout>>,
    aux_vecs: Vec<(String, Type)>,
    nodes: Nodes,
    scanned_files: usize,
}

pub struct Explorer {
    pub nodes: Nodes,
    pub filter_mode: FilterMode,
    folder_filter: Vec<String>,
    folder_filter_i: usize,
    pub grep: RipGrep,
    pub files_scanned: usize,
    pub scanning: bool,
}

impl Explorer {
    pub fn new(search_term: String, folder: String) -> Self {
        let mut grep = RipGrep::new(search_term, folder);
        grep.start_stream();
        Self {
            folder_filter: vec![String::from("")],
            folder_filter_i: 0,
            filter_mode: FilterMode::Contain,
            nodes: Nodes::default(),
            grep,
            files_scanned: 0,
            scanning: true,
        }
    }

    pub fn get_folder_filter_string(&self) -> String {
        self.folder_filter.join(" ")
    }

    pub fn update_folder_filter(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(c) => { self.folder_filter[self.folder_filter_i].push(c); },
            KeyCode::Enter => {
                self.folder_filter.push(String::new());
                self.folder_filter_i += 1;
            },
            KeyCode::Backspace => {
                let ff = &mut self.folder_filter[self.folder_filter_i];

                if !ff.is_empty() {
                    ff.pop();
                } else if self.folder_filter_i > 0 {
                    self.folder_filter.pop();
                    self.folder_filter_i -= 1;
                }
            },
            _ => {}
        }
    }

    pub fn run_wrapper(&mut self) {
        if self.scanning {
            // Drain more lines from the streaming parser, updating nodes incrementally.
            if let Some((nodes, scanned)) = self.grep.drain() {
                self.nodes = nodes;
                self.files_scanned = scanned;
                self.scanning = false;
            }
        } else if self.grep.search_term != self.grep.search_term_buffer {
            self.files_scanned = 0;
            self.scanning = self.grep.start_stream();
        }
    }

    pub fn filtered_nodes(&self) -> Vec<&Node> {
        let items = &self.nodes.0;
        match self.filter_mode {
            FilterMode::Contain => {
                items.iter().filter(|node| node.include_filter(&self.folder_filter)).collect()
            },
            FilterMode::Omit => {
                items.iter().filter(|node| !node.include_filter(&self.folder_filter)).collect()
            }
        }
    }
}

impl RipGrep {
    pub fn new(search_term: String, folder: String) -> Self {
        let after_context = 1;
        let before_context = 1;
        Self {
            search_term_buffer: search_term.clone(),
            search_term,
            after_context, before_context, folder,
            reader: None,
            aux_vecs: vec![],
            nodes: Nodes::default(),
            scanned_files: 0,
        }
    }

    fn rg_args(&self) -> String {
        let (search_term, after_context, before_context, folder) = (&self.search_term, &self.after_context, &self.before_context, &self.folder);
        let threads = num_cpus::get();
        format!("{search_term} --json -A {after_context} -B {before_context} -j {threads} {folder}")
    }

    pub fn start_stream(&mut self) -> bool {
        self.search_term = self.search_term_buffer.clone();
        let args = self.rg_args();
        let child = match Command::new("rg")
            .args(args.split(' '))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return false,
        };
        let stdout = match child.stdout {
            Some(s) => s,
            None => return false,
        };
        self.reader = Some(BufReader::new(stdout));
        self.aux_vecs = vec![];
        self.nodes = Nodes::default();
        self.scanned_files = 0;
        true
    }

    /// Reads more lines from the spawned `rg` process, parsing nodes as they
    /// complete. Returns `Some((nodes, scanned))` when the process has exited
    /// and all lines have been consumed, or `None` if still streaming.
    pub fn drain(&mut self) -> Option<(Nodes, usize)> {
        let reader = match self.reader.as_mut() {
            Some(r) => r,
            None => return Some((self.nodes.clone(), self.scanned_files)),
        };

        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF: process finished
                    self.reader = None;
                    if !self.aux_vecs.is_empty() {
                        if let Some(n) = Node::new(self.aux_vecs.clone(), self.after_context, self.before_context) {
                            self.nodes.0.push(n);
                        }
                        self.aux_vecs = vec![];
                    }
                    self.nodes.0.sort_by_key(|a| a.file_name());
                    return Some((self.nodes.clone(), self.scanned_files));
                }
                Ok(_) => {
                    let trimmed = line.trim_end();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let t = match Nodes::parse_type(trimmed) {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    match t.r#type {
                        Type::begin => {
                            if serde_json::from_str::<crate::explorer::nodes::node::SubnodeBegin>(trimmed).is_ok() {
                                self.scanned_files += 1;
                            }
                        }
                        Type::end => {
                            self.aux_vecs.push((trimmed.to_string(), t.r#type));
                            if let Some(n) = Node::new(self.aux_vecs.clone(), self.after_context, self.before_context) {
                                self.nodes.0.push(n);
                            }
                            self.aux_vecs = vec![];
                        }
                        _ => {
                            self.aux_vecs.push((trimmed.to_string(), t.r#type));
                        }
                    }
                }
                Err(_) => {
                    self.reader = None;
                    return Some((self.nodes.clone(), self.scanned_files));
                }
            }
        }
    }
}

impl Display for RipGrep {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let (search_term, after_context, before_context, folder) = (&self.search_term, &self.after_context, &self.before_context, &self.folder);
        let threads = num_cpus::get();
        write!(f, "rg {search_term} --json -A {after_context} -B {before_context} -j {threads} {folder}")
    }
}

impl Display for Explorer {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.grep)
    }
}


