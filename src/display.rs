
use std::time::Duration;

use tui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
    backend::Backend,
    style::{Style, Color, Modifier},
    text::Span,
    Terminal
};

use crossterm::event::{KeyCode, Event, self, KeyEventKind};

use crate::task::{TaskItem, Task, TaskCategory};

enum InputState {
    None,
    TypeSelect,
    TaskInput(String),
    CategoryInput(String)
}

struct TodoList {
    state: ListState,
    header_task: TaskItem,
    typing_state: InputState
}

impl TodoList {
    fn with_header_item(header_task: TaskItem) -> Self {
        Self { state: ListState::default(), header_task, typing_state: InputState::None }
    }

    fn interact(&mut self) {
        if let Some(i) = self.state.selected() {
            let item = self.header_task.item_at_line_number(i).unwrap();

            match &self.typing_state {
                InputState::TaskInput(s) => {
                    // submit new task
                    let task = Task { name: s.clone(), done: false };
                    // if selected and typing, this should always be a category
                    if let TaskItem::TaskCategory(c) = item {
                        c.child.push(TaskItem::Task(task))
                    }
                },
                InputState::CategoryInput(s) => {
                    // submit new category
                    let category = TaskCategory { name: s.clone(), child: vec![] };
                    // if selected and typing, this should always be a category
                    if let TaskItem::TaskCategory(c) = item {
                        c.child.push(TaskItem::TaskCategory(category))
                    }
                },
                _ => {
                    item.interact()
                }
            }
        };
        self.cancel_typing();
    }

    fn add(&mut self) {
        if let Some(_) = self.state.selected() {
            self.typing_state = InputState::TypeSelect;
        }
    }

    fn cancel_typing(&mut self) {
        self.typing_state = InputState::None
    }

    fn remove(&mut self) {
        if let Some(i) = self.state.selected() {
            let parent = self.header_task.parent_of_line_number(i);
            if let Some(parent) = parent {
                if let TaskItem::TaskCategory(c) = parent.0 {
                    c.child.remove(parent.1);
                }
            }
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.header_task.line_length() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.cancel_typing();
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.header_task.line_length() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.cancel_typing();
    }

    fn unselect(&mut self) {
        self.state.select(None);
        self.cancel_typing();
    }
}


pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, header_task: TaskItem, name: String) -> std::io::Result<TaskItem> {
    let mut todo_list = TodoList::with_header_item(header_task);

    loop {
        terminal.draw(|f| ui(f, &mut todo_list, &name))?;

        if crossterm::event::poll(Duration::from_secs(0))? {
            if let Event::Key(key) = event::read()? {
                match key.kind {
                    KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(todo_list.header_task),
                            KeyCode::Left => {
                                match todo_list.typing_state {
                                    InputState::TypeSelect => todo_list.typing_state = InputState::TaskInput(String::new()),
                                    _ => todo_list.unselect()
                                }
                            },
                            KeyCode::Right => {
                                if let InputState::TypeSelect = todo_list.typing_state {
                                    todo_list.typing_state = InputState::CategoryInput(String::new())
                                }
                            },
                            KeyCode::Down => todo_list.next(),
                            KeyCode::Up => todo_list.previous(),
                            KeyCode::Enter => todo_list.interact(),
                            KeyCode::Char(c) => {
                                // if typing, type the char
                                // otherwise treat as command

                                if let InputState::CategoryInput(s) | InputState::TaskInput(s) = &mut todo_list.typing_state {
                                    s.push(c)
                                } else {
                                    match c {
                                        '-' => todo_list.remove(),
                                        '=' => todo_list.add(),
                                        '+' => todo_list.add(),
                                        _ => ()
                                    }
                                }
                            },
                            KeyCode::Backspace => {
                                if let InputState::CategoryInput(s) | InputState::TaskInput(s) = &mut todo_list.typing_state {
                                    // typing new task/category
                                    s.pop();
                                }
                            }
                            _ => {}
                        }
                    },
                    _ => ()
                }
            }
        }

    }
}

fn ui<B: Backend>(f: &mut Frame<B>, todo_list: &mut TodoList, name: &str) {
    let rect = f.size();

    let mut all_line_strings = Vec::new();
    todo_list.header_task.all_text_lines(&mut all_line_strings, 0);

    let selected = todo_list.state.selected().unwrap_or(99999);

    let lines: Vec<ListItem> = all_line_strings.iter_mut().enumerate().map(|(i, x)| {
        if i == selected {
            // cheat to check if line is task or category
            if x.contains('☐') || x.contains('☑') {
                x.push_str("        (↰ / -)")
            } else {
                match &todo_list.typing_state {
                    InputState::TypeSelect => {
                        x.push_str("       ◀ Task : Category ▶")
                    },
                    InputState::TaskInput(s) => {
                        x.push_str("    New Task: ");
                        x.push_str(s);
                        x.push('_');
                    },
                    InputState::CategoryInput(s) => {
                        x.push_str("    New Category: ");
                        x.push_str(s);
                        x.push('_');
                    },
                    InputState::None => {
                        x.push_str("        (+ / -)")
                    }
                }
            }
        }

        let line = Span::from(x.as_str());
        ListItem::new(line)
    }).collect();

    let items = List::new(lines)
        .block(Block::default().borders(Borders::ALL).title(name))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(items, rect, &mut todo_list.state);
}
