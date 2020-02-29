#[macro_use] extern crate serde_derive;
#[macro_use] extern crate clap;

use clap::{Arg, App, SubCommand};

use std::process;
use std::env;

use std::fs::{self, File};

use std::io::{Error, ErrorKind};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use std::time::SystemTime;

use chrono::{DateTime, TimeZone, Utc, Local};

use colored::*;

pub static TASKS_FILE: &'static str = "todoclist.json";

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    creation: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed: Option<u64>,
}

impl Task {
    fn new(name: String) -> Task {
        Task {name: name, creation: timestamp(), author: None, completed: Some(0)}
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct List {
    tasks: Vec<Task>,
}

impl List {
    fn new() -> List {
        List {tasks: Vec::new()}
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    fn init<T: AsRef<Path>>(&self, file_path: T) -> std::io::Result<()> {
        match fs::metadata(&file_path) {
            Ok(_) => return Err(Error::new(ErrorKind::AlreadyExists, format!("\"{}\" already exsits!", TASKS_FILE))),
            Err(_) => {},
        };

        self.save(file_path)
    }

    fn save<T: AsRef<Path>>(&self, file_path: T) -> std::io::Result<()> {
        let contents = serde_json::to_string(&self)?;

        let mut file = File::create(file_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

pub fn timestamp() -> u64{
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

fn file_name() -> std::io::Result<String> {
    let path_buf = env::current_exe()?;
    match path_buf.file_name() {
        Some(name) => match name.to_str() {
            Some(string) => Ok(String::from(string)),
            None => Err(Error::new(ErrorKind::Other, "no str"))
        },
        None => Err(Error::new(ErrorKind::Other, "nostr"))
    }
}

fn canonical<T: AsRef<Path>>(value: T) -> PathBuf {
    let root = env::current_dir().unwrap();
    absoluteify(root, value)
}

fn absoluteify<A, B>(root: A, value: B) -> PathBuf
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let root = root.as_ref();
    let value = value.as_ref();

    if value.is_absolute() {
        PathBuf::from(value)
    } else {
        root.join(value)
    }
}

fn init_list(absolute_path: &PathBuf) -> Option<List> {
    let list = List::new();
    match list.init(absolute_path) {
        Ok(_) => println!("Initialized new todolist at {:?}", absolute_path),
        Err(e) => {
            eprintln!("Error initializing the todolist: {}", e);
            return None;
        },
    };
    Some(list)
}

fn get_list(absolute_path: &PathBuf, auto_init: bool) -> Option<List> {
    if fs::metadata(absolute_path).is_ok() {
        let mut file = File::open(absolute_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let list: List = serde_json::from_str(&contents).unwrap();
        return Some(list);
    }

    if !auto_init {
        eprintln!("File {} doesn't exists, please use the \"init\" subcommand or the \"--auto-init\" flag", TASKS_FILE);
        return None;
    }

    init_list(absolute_path)
}

fn pretty_task(task: (usize, Task)) -> String {
    let local_date: DateTime<Local> = DateTime::from(Utc.timestamp(task.1.creation as i64, 0));

    let mut output = String::new();

    output.push_str(&(format!("#{}: ", task.0+1).green().bold().to_string()));
    output.push_str(&(task.1.name.cyan().bold().to_string()));
    output.push_str(&"\n \\-> Created at ".magenta().bold().to_string());
    output.push_str(&(format!("{}", local_date.format("%a %b %e %r %Y")).yellow().bold().to_string()));

    output
}

fn match_id(str_id: &str) -> usize {
    let id: usize = match str_id.parse() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("The id should be numeric");
            process::exit(1);
        },
    };

    match id {
        0 => {
            eprintln!("Task #{} doesn't exists", id);
            process::exit(1);
        },
        _ => id -1,
    }
}

fn main() {
    let exe_name = file_name().unwrap_or(String::from("a"));
    let matches = App::new("TodoCLIst")
    .author(crate_authors!())
    .version(crate_version!())
    .about("A simple file-cli-based todolist")
    .subcommand(SubCommand::with_name("init")
        .about("Initialize a project")
    )
    .subcommand(SubCommand::with_name("add")
        .about("Add a new task")
        .arg(Arg::with_name("description")
            .required(true)))
    .subcommand(SubCommand::with_name("list")
        .about("List all tasks"))
    .subcommand(SubCommand::with_name("complete")
        .about("Complete a task by its id")
        .arg(Arg::with_name("id")
            .required(true)))
    .subcommand(SubCommand::with_name("delete")
        .about("Delete a task by its id")
        .arg(Arg::with_name("id")
            .required(true)))
    .arg(Arg::with_name("init")
        .short("i")
        .long("auto-init")
        .help("Auto initialize the file before any command")
        .global(true))
    .arg(Arg::with_name("path")
        .short("p")
        .long("path")
        .help(&format!("Path to the directory containing {}", TASKS_FILE))
        .takes_value(true)
        .global(true))
    .get_matches();


    let absolute_path = canonical(match matches.value_of("path") {
        Some(path) => path,
        None => "."
    }).join(TASKS_FILE);

    let auto_init = match matches.occurrences_of("init") {
        0 => false,
        _ => true,
    };

    match matches.subcommand() {
        ("init", _) => {
            init_list(&absolute_path);
        }
        ("add", sub_matches) => {
            let sub_matches = sub_matches.unwrap();

            let description = sub_matches.value_of("description").unwrap();

            match get_list(&absolute_path, auto_init) {
                Some(mut list) => {
                    list.add_task(Task::new(String::from(description)));
                    match list.save(&absolute_path) {
                        Ok(_) => println!("Added task \"{}\" with id #{}", description, list.tasks.len()),
                        Err(e) => eprintln!("Error saving task: {}", e),
                    }
                },
                None => {}
            };
        }
        ("list", _) => {
            match get_list(&absolute_path, auto_init) {
                Some(list) => {
                    let mut iter = list.tasks.into_iter().enumerate().filter(|(_, task)| task.completed.is_none()).peekable();
                    match iter.peek() {
                        Some(_) => iter.map(pretty_task).for_each(|s| println!("{}", s)),
                        None => println!("List empty, good job!"),
                    }
                    
                },
                None => {}
            }
        },
        ("complete", sub_matches) => {
            let sub_matches = sub_matches.unwrap();
            let str_id = sub_matches.value_of("id").unwrap();
            let id = match_id(str_id);

            match get_list(&absolute_path, auto_init) {
                Some(mut list) => {
                    match list.tasks.get_mut(id) {
                        Some(mut task) => {
                            task.completed = Some(timestamp());
                            match list.save(&absolute_path) {
                                Ok(_) => println!("Completed task #{}, very nice!", id+1),
                                Err(e) => eprintln!("Error completing task: {}", e),
                            }
                        },
                        None => eprintln!("Task #{} doesn't exists", id+1)
                    }
                },
                None => {}
            };
        },
        ("delete", sub_matches) => {
            let sub_matches = sub_matches.unwrap();
            let str_id = sub_matches.value_of("id").unwrap();
            let id = match_id(str_id);

            match get_list(&absolute_path, auto_init) {
                Some(mut list) => {
                    let task = list.tasks.remove(id);
                    match list.save(&absolute_path) {
                        Ok(_) => println!("Deleted task #{}: {}", id+1, task.name),
                        Err(e) => eprintln!("Error deleting task: {}", e),
                    }
                },
                None => {}
            };
        },
        _ => {
            eprintln!("Invalid subcommand, please use '{} help'", exe_name);
            process::exit(1);
        }
    }
}