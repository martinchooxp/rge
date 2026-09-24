use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

use std::io;

use tui::{
    backend::CrosstermBackend,
    Terminal
};
use clap::Parser;

mod explorer;
mod ui;
mod wrapper;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    search_term: String,
    folder: Option<String>,
    #[arg(long)]
    debug: bool,
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.debug {
        eprintln!("[DEBUG] CLI parsed: search_term={}, folder={:?}, debug=true", cli.search_term, cli.folder);
    }

    enable_raw_mode().expect("can run in raw mode");

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let folder = if let Some(f) = cli.folder { f } else { String::from(".") };

    wrapper::explorer_wrapper(&mut terminal, cli.search_term, folder, cli.debug)?;

    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}