use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    thread,
    time::Duration,
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
struct Entries {
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
        match self.s.selected() {
            Some(_) => self.s.select(None),
            None => (),
        };
    }
    pub fn enter(&mut self) {
        match self.s.selected() {
            Some(i) => match self.v[i].t {
                EntryType::Dir => self.p.push(self.v[i].p.file_name().unwrap()), // Here we shouldnt modify the self as a whole, but rather change the table,
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
            },
            None => (),
        };
        self.update();
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
    fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let rect = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area)[0];
        let selected_style = Style::default()
            .bg(Color::LightRed)
            .add_modifier(Modifier::ITALIC);
        let normal_style = Style::default().bg(Color::LightRed);
        let current_dir = self.p.to_str().unwrap();
        let header_cells = ["File"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let header = Row::new([current_dir])
            .style(normal_style.fg(Color::White))
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
fn entries_ui<B: Backend>(f: &mut Frame<B>, entries: &mut Entries, area: Rect) {
    let rect = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area)[0];
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["File"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    let rows: Vec<Row<'_>> = entries.v.iter().map(|f| f.as_row()).collect();
    let widget = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol("-> ")
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Max(30),
            Constraint::Min(20),
        ]);
    f.render_stateful_widget(widget, rect, &mut entries.s);
}
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
// fn strify_files<'a>(path: impl AsRef<Path>) -> io::Result<Vec<Cow<'a, str>>> {
//     let mut c = vec![];
//     let names = recurse_files(path)?.to_owned();
//     names.iter().for_each(|e| c.push(e.to_string_lossy()));
//     // .for_each(|e| )
//     // .map(|e| e.to_string_lossy())
//     // .collect();
//     Ok(c)
// }
struct FileTable<'a> {
    state: TableState,
    items: Vec<Row<'a>>,
}
impl<'a> FileTable<'a> {
    fn new() -> Self {
        Self {
            state: TableState::default(),
            items: vec![Row::new(vec!["C1", "C2"])],
        }
    }
    fn from_dir(path: &'a str) -> Self {
        let mut rows = vec![Row::new(vec!["..", ""])];
        // std::fs::read_dir(path)
        match std::fs::read_dir(path) {
            Ok(dir) => dir.filter_map(|e| e.ok()).for_each(|e| {
                rows.push(Row::new(vec![
                    e.file_name().into_string().unwrap(),
                    0u64.to_string(), // u64::to_string(&e.metadata().unwrap()),
                ]))
            }),
            Err(_) => (),
        };
        Self {
            state: TableState::default(),
            items: rows,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }
}
fn really_run_app<B: Backend>(terminal: &mut Terminal<B>, mut entries: Entries) -> io::Result<()> {
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
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut ft: FileTable) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let screensize = f.size();
            let ft_box = Rect {
                x: screensize.x,
                y: screensize.y,
                width: screensize.width / 4,
                height: screensize.height,
            };
            ft_ui(f, &mut ft, ft_box)
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => ft.next(),
                    KeyCode::Up => ft.previous(),
                    _ => {}
                }
            }
        }
    }
}
// example code to work off of
fn ft_ui<B: Backend>(f: &mut Frame<B>, ft: &mut FileTable, ui_box: Rect) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(ui_box);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["File", "Size"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    // let rows = ft.items.iter().map(|item| {
    //     let height = item
    //         .iter()
    //         .map(|content| content.chars().filter(|c| *c == '\n').count())
    //         .max()
    //         .unwrap_or(0)
    //         + 1;
    //     let cells = item.iter().map(|c| Cell::from(c.clone()));
    //     Row::new(cells).height(height as u16).bottom_margin(1)
    // });
    let rows = ft.items.clone();
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol("-> ")
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Max(30),
            Constraint::Min(20),
        ]);
    f.render_stateful_widget(t, rects[0], &mut ft.state);
}
fn main() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let dir_to_search = std::env::current_dir().unwrap();
    let ft = FileTable::from_dir(dir_to_search.to_str().unwrap());
    let entries = Entries::new(dir_to_search)?;
    let r = really_run_app(&mut terminal, entries);
    // let res = run_app(&mut terminal, ft);

    // terminal.draw(|f| {
    //     let size = f.size();
    //     let block = Block::default().title("Block").borders(Borders::ALL);
    //     f.render_widget(block, size);
    // })?;

    // Start a thread to discard any input events. Without handling events, the
    // stdin buffer will fill up, and be read into the shell when the program exits.
    // thread::spawn(|| loop {
    //     let _e = event::read().unwrap();
    // });

    // thread::sleep(Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = r {
        println!("{e:?}");
    }
    Ok(())
}
