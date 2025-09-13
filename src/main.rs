mod editor;

use editor::Editor;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::env;
use std::io::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("untitled.txt");

    let mut editor = Editor::new();
    editor.open(filename);

    enable_raw_mode()?;
    loop {
        editor.refresh_screen()?;
        if !editor.process_keypress()? {
            break;
        }
    }
    disable_raw_mode()?;
    Ok(())
}
