use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
#[derive(Clone)]
enum EntryType {
    Dir,
    File,
    Sym,
}
impl EntryType {
    fn style(&self) -> Style {
        match self {
            EntryType::Dir => Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue),
            EntryType::File => Style::default(),
            EntryType::Sym => Style::default().fg(Color::LightGreen),
        }
    }
}
#[derive(Clone)]
struct Entry {
    p: PathBuf,
    t: EntryType,
}
impl Entry {
    pub fn new(d: fs::DirEntry) -> io::Result<Self> {
        let md = d.metadata()?;
        let mut t = EntryType::File;
        if md.is_dir() {
            t = EntryType::Dir;
        } else if md.is_symlink() {
            t = EntryType::Sym;
        }
        Ok(Self { p: d.path(), t })
    }
    pub fn as_cell(&self) -> Cell {
        Cell::from(self.p.file_name().unwrap().to_str().unwrap()).style(self.t.style())
    }
    pub fn as_row(&self) -> Row {
        Row::new([self.as_cell()])
        // Row::new([Cell::from(" "), self.as_cell()])
    }
}
pub struct Entries {
    p: PathBuf,
    s: TableState,
    v: Vec<Entry>,
}
impl FromIterator<Entry> for Entries {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        let mut entries = vec![];
        iter.into_iter().for_each(|e| entries.push(e));
        Self {
            p: entries[0].p.parent().unwrap().to_path_buf(),
            s: TableState::default(),
            v: entries,
        }
    }
}
impl Entries {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            p: path.as_ref().to_path_buf(),
            s: TableState::default(),
            v: fs::read_dir(path)?
                .filter_map(|d| d.ok())
                .filter_map(|e| Entry::new(e).ok())
                .collect(),
        })
    }
    pub fn update(&mut self) {
        *self = Self::new(self.p.clone()).unwrap();
    }
    pub fn as_table(&mut self) -> Table {
        let mut rows = vec![];
        self.v.iter().for_each(|e| rows.push(e.as_row()));
        Table::new(rows)
    }
    pub fn get_state(&mut self) -> &mut TableState {
        &mut self.s
    }
    pub fn next(&mut self) {
        let i = match self.s.selected() {
            Some(i) => {
                if i >= self.v.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.s.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.s.selected() {
            Some(i) => {
                if i == 0 {
                    self.v.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.s.select(Some(i))
    }
    pub fn deselect(&mut self) {
        if self.s.selected().is_some() {
            self.s.select(None)
        }
        // if let Some(_) = self.s.selected() {
        //     self.s.select(None)
        //     // match self.v[selected].t {

        //     // }
        // };
        // match self.s.selected() {
        //     Some(_) => self.s.select(None),
        //     None => (),
        // };
    }
    pub fn enter(&mut self) {
        if let Some(selected) = self.s.selected() {
            match self.v[selected].t {
                EntryType::Dir => self.p.push(self.v[selected].p.file_name().unwrap()), // Here we shouldnt modify the self as a whole, but rather change the table,
                // Ideal would be:
                // File
                // Dir
                // =>
                // File
                // Dir -> File/Dir
                // ... -> File/Dir
                // ... -> ... Etc
                EntryType::File => todo!(), // Make the file content render rather in this case i think
                EntryType::Sym => todo!(), // Handle like file but maybe add functionality to do some subview to peek into the linked directory?
            };
            self.update();
        };
        // match self.s.selected() {
        //     Some(i) => match self.v[i].t {
        //         EntryType::Dir => self.p.push(self.v[i].p.file_name().unwrap()), // Here we shouldnt modify the self as a whole, but rather change the table,
        //         // Ideal would be:
        //         // File
        //         // Dir
        //         // =>
        //         // File
        //         // Dir -> File/Dir
        //         // ... -> File/Dir
        //         // ... -> ... Etc
        //         EntryType::File => todo!(), // Make the file content render rather in this case i think
        //         EntryType::Sym => todo!(), // Handle like file but maybe add functionality to do some subview to peek into the linked directory?
        //     },
        //     None => (),
        // };
        // self.update();
    }
    pub fn ret(&mut self) {
        self.p.pop();
        self.update();
        // *self = Self::new(self.p.parent().unwrap()).unwrap()
    }
    pub fn header(&self) -> Row {
        let header_cells = ["File", self.p.to_str().unwrap()];
        let header = Row::new(header_cells);
        header
    }
    pub(crate) fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let rect = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area)[0];
        let selected_style = Style::default()
            .bg(Color::LightBlue)
            .fg(Color::Black)
            .add_modifier(Modifier::ITALIC);
        let normal_style = Style::default().bg(Color::LightBlue);
        let current_dir = self.p.to_str().unwrap();
        // let header_cells = ["File"]
        //     .iter()
        //     .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let header = Row::new([current_dir])
            .style(
                normal_style
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
            )
            .height(1)
            .bottom_margin(0);
        let rows: Vec<Row<'_>> = self.v.iter().map(|f| f.as_row()).collect();
        let widget = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Table"))
            .highlight_style(selected_style)
            .highlight_symbol(" --> ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .widths(&[
                // Constraint::Max(4),
                Constraint::Percentage(96),
                // Constraint::Min(5),
            ]);
        f.render_stateful_widget(widget, rect, &mut self.s);
    }
}
