use crossterm::event::KeyCode;
use std::fs::OpenOptions;
use std::io::{Stdout, Write};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
    widgets::ListState,
    Terminal,
};
use crate::ui;
use crate::ui::nodes::{action_nodes, render_nodes};
use crate::explorer::Explorer;

fn log_debug(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .append(true)
        .create(true)
        .open("debug.txt")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

fn open_editor(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    file_path: &str,
    line_number: usize,
    search_term: &str,
    debug: bool,
) {
    let editor_raw = std::env::var("EDITOR").unwrap_or_else(|_| String::from("nvim"));
    let editor_raw = editor_raw.trim();
    // Ignore omarchy-launch-editor (with or without --inline) and use plain nvim
    let editor_raw = if editor_raw.starts_with("omarchy-launch-editor") {
        String::from("nvim")
    } else {
        editor_raw.to_string()
    };

    let mut editor_parts = editor_raw.split_whitespace();
    let editor_cmd = editor_parts.next().unwrap_or("nvim");

    if debug {
        log_debug(&format!("[DEBUG] Opening editor '{}' on file '{}' at line {} search='{}'", editor_cmd, file_path, line_number, search_term));
    }

    let mut cmd = std::process::Command::new(editor_cmd);
    cmd.args(editor_parts);

    // For vim/nvim: preload the search term so the editor opens pre-searching
    // for the search term (coincidence) argument.
    let is_vim_like = editor_cmd.contains("vim")
        || editor_cmd.contains("nvim")
        || editor_cmd.contains("vi");
    if is_vim_like && !search_term.is_empty() {
        cmd.arg(format!("+/{}", search_term));
    }

    if editor_cmd.contains("hx") || editor_cmd.contains("helix") {
        cmd.arg(format!("{}:{}", file_path, line_number));
    } else {
        if editor_cmd.contains("nano")
            || editor_cmd.contains("vim")
            || editor_cmd.contains("nvim")
            || editor_cmd.contains("vi")
            || editor_cmd.contains("emacs")
        {
            cmd.arg(format!("+{}", line_number));
        }
        cmd.arg(file_path);
    }

    if debug {
        log_debug("[DEBUG] Disabling raw mode and showing cursor...");
    }
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = terminal.show_cursor();
    let _ = cmd.status();
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = terminal.hide_cursor();
    let _ = terminal.clear();
}

pub fn explorer_wrapper(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    search_term: String,
    folders: String,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut explorer = Explorer::new(search_term.clone(), folders);
    let mut app = ui::App::default();
    let mut node_list_state = ListState::default();
    node_list_state.select(Some(0));

    if debug {
        log_debug("[DEBUG] explorer_wrapper started");
    }

    loop {
        explorer.run_wrapper();
        let filtered_nodes = explorer.filtered_nodes();

        // Clamp selected index to valid range
        let max_idx = filtered_nodes.len().saturating_sub(1);
        let current = node_list_state.selected().unwrap_or(0);
        let clamped = if filtered_nodes.is_empty() {
            None
        } else {
            Some(current.min(max_idx))
        };
        if clamped != node_list_state.selected() {
            node_list_state.select(clamped);
        }

        let selected_node = filtered_nodes
            .get(clamped.unwrap_or(0))
            .copied();

        if debug {
            log_debug(&format!("[DEBUG] Filtered nodes count: {}", filtered_nodes.len()));
        }

        terminal.draw(|rect| {
            if filtered_nodes.is_empty() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("No results")
                    .border_type(BorderType::Plain);
                rect.render_widget(block, rect.size());
                return;
            }
            let nodes_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(rect.size());
            let (left, right) = render_nodes(&filtered_nodes, selected_node, &app, &explorer);
            rect.render_stateful_widget(left, nodes_chunks[0], &mut node_list_state);
            rect.render_widget(right, nodes_chunks[1]);
        })?;

        let event = crossterm::event::read()?;
        if debug {
            log_debug(&format!("[DEBUG] Event read: {:?}", event));
        }

        if let crossterm::event::Event::Key(key) = event {
            if debug {
                log_debug(&format!("[DEBUG] Key pressed: {:?} modifiers: {:?}", key.code, key.modifiers));
            }
            match app.get_input_mode() {
                ui::InputMode::Normal => {
                    if key.code == KeyCode::Char('q')
                        || (key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('c'))
                    {
                        clearscreen::clear().expect("Failed to clear screen");
                        break;
                    }
                    if key.code == KeyCode::Enter || key.code == KeyCode::Right {
                        if let Some(node) = selected_node {
                            let file_path = node.file_name();
                            let line_number = node
                                .line_number_at(app.offset_detail)
                                .or_else(|| node.line_number_at(0))
                                .unwrap_or(1);
                            if debug {
                                log_debug(&format!("[DEBUG] Launching editor: file={}, line={}", file_path, line_number));
                            }
                            open_editor(terminal, &file_path, line_number, &search_term, debug);
                            continue;
                        } else if debug {
                            log_debug("[DEBUG] No selected node found for editor launch.");
                        }
                    }
                }
                ui::InputMode::Editing => {
                    if key.code == KeyCode::Esc || key.code == KeyCode::F(2) {
                        app.set_input_mode(ui::InputMode::Normal);
                        continue;
                    }
                }
            }
            action_nodes(
                &mut explorer,
                &mut app,
                key,
                &mut node_list_state,
            );
        }
    }
    Ok(())
}