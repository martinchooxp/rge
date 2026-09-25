use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str;
use std::process::{Command, Stdio};

use crossterm::event::KeyCode;

use crate::ui::FilterMode;
use crate::explorer::nodes::Nodes;
use crate::explorer::nodes::Node;

pub mod nodes;

pub struct RipGrep {
    search_term: String,
    pub search_term_buffer: String,
    after_context: usize,
    before_context: usize,
    folder: String,
}

pub struct Explorer {
    pub nodes: Nodes,
    pub filter_mode: FilterMode,
    folder_filter: Vec<String>,
    folder_filter_i: usize,
    pub grep: RipGrep,
}

impl Explorer {
    pub fn new(search_term: String, folder: String) -> Self {
        let mut grep = RipGrep::new(search_term, folder);
        let nodes = grep.run();
        Self {
            folder_filter: vec![String::from("")],
            folder_filter_i: 0,
            filter_mode: FilterMode::Contain,
            nodes, grep,
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
        if self.grep.search_term != self.grep.search_term_buffer {
            self.nodes = self.grep.run();
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
        }
    }

    fn launch_rg(arguments: String) -> Option<String> {
        let arguments = arguments.split(' ');
        let child = match Command::new("rg")
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return None,
        };
        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return None,
        };

        let s = match str::from_utf8(&output.stdout) {
            Ok(v) => v,
            Err(_) => return None,
        };
        if s.is_empty() { return None; }
        Some(Self::strip_trailing_newline(s).to_string())
    }

    fn rg_args(&self) -> String {
        let (search_term, after_context, before_context, folder) = (&self.search_term, &self.after_context, &self.before_context, &self.folder);
        format!("{search_term} --json -A {after_context} -B {before_context} {folder}")
    }

    fn run(&mut self) -> Nodes {
        self.search_term = self.search_term_buffer.clone();
        let args = self.rg_args();
        let res = Self::launch_rg(args);
        match res {
            Some(res) => {
                let res = res.split("\n").collect::<Vec<&str>>();
                Nodes::new(res, self.after_context, self.before_context)
            },
            None => {
                Nodes::new(vec![], self.after_context, self.before_context)
            }
        }
    }

    fn strip_trailing_newline(input: &str) -> &str {
        input
            .strip_suffix("\r\n")
            .or(input.strip_suffix("\n"))
            .unwrap_or(input)
    }
}

impl Display for RipGrep {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let (search_term, after_context, before_context, folder) = (&self.search_term, &self.after_context, &self.before_context, &self.folder);
        write!(f, "rg {search_term} --json -A {after_context} -B {before_context} {folder}")
    }
}

impl Display for Explorer {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.grep)
    }
}


