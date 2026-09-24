use crossterm::event::{KeyCode, KeyEvent};

use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table,
    },
};

use crate::ui::NodeTabSelected;
use crate::explorer::nodes::Node;
use crate::ui::{App, InputMode, FilterMode};

pub fn render_nodes<'a>(
    filtered_nodes: &[&'a Node],
    selected_node: Option<&'a Node>,
    app: &App,
    explorer: &crate::explorer::Explorer,
) -> (List<'a>, Table<'a>) {
    let selected_node_tab = &app.selected_node_tab;
    let (style_list, style_detail) = match selected_node_tab {
        NodeTabSelected::Detail => {
            (Style::default().fg(Color::Gray), Style::default().fg(Color::White))
        }
        NodeTabSelected::FileList => match app.selected_node_tab {
            NodeTabSelected::FileList => {
                (Style::default().fg(Color::Gray), Style::default().fg(Color::Gray))
            }
            _ => (Style::default().fg(Color::Gray), Style::default().fg(Color::Gray)),
        }
    };

    let title = if explorer.scanning {
        format!("Filter: '{}' (scanning {} files...)", explorer.get_folder_filter_string(), explorer.files_scanned)
    } else {
        format!("Filter: '{}'", explorer.get_folder_filter_string())
    };
    let nodes_block: Block = Block::default()
        .borders(Borders::ALL)
        .style(style_list)
        .title(title)
        .border_type(BorderType::Plain);

    let items: Vec<ListItem> = filtered_nodes
        .iter()
        .map(|node| {
            ListItem::new(Spans::from(vec![Span::styled(
                node.summary(),
                Style::default(),
            )]))
        })
        .collect();

    let list = List::new(items)
        .block(nodes_block)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let (file_name, node_detail) = if let Some(node) = selected_node {
        (node.file_name(), node.detail(app.offset_detail))
    } else {
        (String::from(""), Table::new(vec![]))
    };

    let node_detail = node_detail
        .header(Row::new(vec![Cell::from(Span::styled(
            format!(" {}", file_name),
            Style::default().add_modifier(Modifier::BOLD),
        ))]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(style_detail)
                .title("Detail")
                .border_type(BorderType::Plain),
        )
        .widths(&[Constraint::Percentage(100)]);

    (list, node_detail)
}

pub fn action_nodes(
    explorer: &mut crate::explorer::Explorer,
    app: &mut App,
    key: KeyEvent,
    node_list_state: &mut ListState,
) {
    match app.get_input_mode() {
        InputMode::Normal => match key.code {
            KeyCode::Char('i') => app.set_input_mode(InputMode::Editing),
            KeyCode::Char('c') => {
                explorer.filter_mode = FilterMode::Contain;
            }
            KeyCode::Char('o') => {
                explorer.filter_mode = FilterMode::Omit;
            }
            _ => {}
        },
        InputMode::Editing => {
            explorer.update_folder_filter(key.code);
        }
    }

    match key.code {
        KeyCode::Down => match app.selected_node_tab {
            NodeTabSelected::FileList => {
                if let Some(selected) = node_list_state.selected() {
                    let amount_nodes = explorer.filtered_nodes().len();
                    if amount_nodes > 0 {
                        if selected >= amount_nodes - 1 {
                            node_list_state.select(Some(0));
                        } else {
                            node_list_state.select(Some(selected + 1));
                        }
                    } else {
                        node_list_state.select(Some(0));
                    }
                }
                app.offset_detail = 0;
            }
            NodeTabSelected::Detail => {
                if let Some(selected) = node_list_state.selected() {
                    if let Some(node) = explorer.filtered_nodes().get(selected) {
                        if app.offset_detail < node.len_matches_all() {
                            app.offset_detail += 1;
                        }
                    }
                }
            }
        },
        KeyCode::Up => match app.selected_node_tab {
            NodeTabSelected::FileList => {
                if let Some(selected) = node_list_state.selected() {
                    let amount_nodes = explorer.filtered_nodes().len();
                    if amount_nodes > 0 {
                        if selected > 0 {
                            node_list_state.select(Some(selected - 1));
                        } else {
                            node_list_state.select(Some(amount_nodes - 1));
                        }
                    } else {
                        node_list_state.select(Some(0));
                    }
                }
                app.offset_detail = 0;
            }
            NodeTabSelected::Detail => {
                if app.offset_detail > 0 {
                    app.offset_detail -= 1;
                }
            }
        },
        KeyCode::Tab => {
            app.selected_node_tab = if app.selected_node_tab == NodeTabSelected::FileList {
                NodeTabSelected::Detail
            } else {
                NodeTabSelected::FileList
            }
        }
        _ => {}
    }
}