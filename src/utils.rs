use crate::browser::Entries;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::Backend, prelude::Rect, Terminal};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
pub(crate) fn really_run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut entries: Entries,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let screensize = f.size();
            let area = Rect {
                x: screensize.x,
                y: screensize.y,
                width: screensize.width / 3,
                height: screensize.height,
            };
            entries.render(f, area);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => entries.deselect(),
                    KeyCode::Down | KeyCode::Char('j') => entries.next(),
                    KeyCode::Up | KeyCode::Char('k') => entries.previous(),
                    KeyCode::Right | KeyCode::Char('l') => entries.enter(),
                    KeyCode::Left | KeyCode::Char('h') => entries.ret(),
                    _ => {}
                }
            }
        }
    }
}
#[allow(dead_code)]
fn recurse_files(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let mut buf = vec![];
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            buf.append(&mut subdir);
        }

        if meta.is_file() {
            buf.push(entry.path());
        }
    }
    Ok(buf)
}
