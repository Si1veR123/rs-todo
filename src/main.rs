mod task;
mod display;

use std::{path::PathBuf, fs::{OpenOptions, create_dir}, io::Read};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute, event::{EnableMouseCapture, DisableMouseCapture}};
use task::{TaskItem, TaskCategory};
use display::run_app;
use tui::{backend::CrosstermBackend, Terminal};
use serde_json::{from_str, to_writer};
use home::home_dir;

const SAVED_DATA_FOLDER: &'static str = "AppData\\Local\\todo_rs\\";


fn file_path(name: Option<String>) -> PathBuf {
    let mut file = name.unwrap_or(String::from("main_list_0"));
    file.push_str(".json");
    home_dir().unwrap().join(SAVED_DATA_FOLDER).join(file)
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argument = std::env::args().nth(1);
    let list_display_name = argument.clone().unwrap_or(String::from("Main"));
    let file_path = file_path(argument);

    let _ = create_dir(file_path.parent().unwrap().clone());

    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(file_path.clone())?;

    let mut file_buf = String::new();
    let _ = file.read_to_string(&mut file_buf);
    let parsed: Result<TaskItem, _> = from_str(&file_buf);
    println!("{:?}", parsed);
    let root_task = parsed.unwrap_or(TaskItem::TaskCategory( TaskCategory { name: String::from("root"), child: vec![] } ));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, root_task, list_display_name);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(task) => {
            let file = OpenOptions::new()
                .write(true)
                .open(file_path)?;
            let _ = file.set_len(0);
            let r = to_writer(file, &task);

            if r.is_err() {
                println!("Error saving list.")
            } else {
                println!("List saved.")
            }
        },
        Err(e) => {
            println!("{}", e);
        }
    }

    Ok(())
}
