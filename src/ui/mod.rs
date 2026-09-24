pub mod nodes;

#[derive(Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq)]
pub enum NodeTabSelected {
    FileList,
    Detail,
}

pub enum FilterMode {
    Contain,
    Omit,
}

pub struct App {
    selected_node_tab: NodeTabSelected,
    pub offset_detail: usize,
    input_mode: InputMode,
}

impl Default for App {
    fn default() -> App {
        App {
            offset_detail: 0,
            selected_node_tab: NodeTabSelected::FileList,
            input_mode: InputMode::Normal,
        }
    }
}

impl App {
    pub fn get_input_mode(&self) -> InputMode {
        self.input_mode
    }
    pub fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }
}