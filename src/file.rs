//! # TodoFile
//! 
//! This module can be used to read and write todo.txt files.
//! It simplifies and abstracts away the complexities by introducing a simple struct that can be
//! used to interact with todo.txt files.
//! Most of the methods in this struct are inspired by the todo.txt cli application.
//! Hence, it is easy and intuitive to use.
//!
//! The basic usage of the struct is as follows:
//! ```rust
//! use libdonow::file::TodoFile;
//!
//! let mut file = TodoFile::new("todo.txt");
//! ```
//!
//! This reads off the file from disk and parses it into a `TodoFile` struct.
//! The parsing is done in the `load` method, so you can call this method to reparse the file.
//!
//! If you are looking for simple handling of todo.txt files, this struct is the way to go.
//! However, for more intricate handling of todo items, you can use the `parser::Todo` struct,
//! which is used internally by this struct.


use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use fancy_regex::Regex;

use crate::parser;

/// A struct that represents a todo.txt file.
/// This struct doesn't actually represent a file on disk, but rather a collection of todos.
/// Any actions on the struct do not affect the file on disk, you have to call the `save` method to
/// save the changes.
pub struct TodoFile {
    /// The path to the file on disk.
    pub path: PathBuf,
    /// A vector of todo items.
    pub todos: Vec<parser::Todo>,
    /// The content of the file as a string.
    pub content: String,
}

impl TodoFile {
    /// Reads off a path to a file and returns a `TodoFile` struct.
    /// If the file doesn't exist, an empty `TodoFile` struct is returned implicitly.
    /// For more granular control flow of the file, you can use the from_path method.
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => String::new(),
        };

        let mut t = TodoFile::from_path(&path).unwrap_or_else(|_| TodoFile {
            path,
            todos: Vec::new(),
            content,
        });
        t.load();

        t
    }

    /// Reads off a path to a file and returns a `TodoFile` struct.
    /// If the file doesn't exist, it returns an IO error.
    /// This method is useful when you want to handle the error explicitly.
    /// Otherwise, you can use the `new` method.
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }

        let content = std::fs::read_to_string(path).unwrap();
        let mut t = TodoFile {
            path: PathBuf::from(path),
            todos: Vec::new(),
            content,
        };
        t.load();

        Ok(t)
    }

    /// Reads off a string and returns a `TodoFile` struct.
    /// This method skips the file reading and directly parses the string into a `TodoFile` struct.
    /// Might be useful if you store the files in a database or some other storage.
    pub fn from_string(content: &str) -> Self {
        let content = content.to_string();
        let mut t = TodoFile {
            path: PathBuf::new(),
            todos: Vec::new(),
            content,
        };
        t.load();

        t
    }

    /// The main function that parses each line of the file and stores it in the `todos` vector.
    /// The line is determined by the `Line` struct as defined by rust's standard library.
    /// Each line is passed into the parse method of the `Todo` struct, which returns a `Result`.
    /// Any errors are ignored and the loop continues.
    pub fn load(&mut self) {
        let lines = self.content.lines();
        let mut todos = Vec::new();

        for line in lines {
            match parser::Todo::parse(line) {
                Ok(todo) => todos.push(todo),
                Err(_) => continue,
            }
        }

        self.todos = todos;
    }

    /// Saves the `TodoFile` struct to the file on disk.
    /// The path has to be set before calling this method.
    /// Alternatively, you can use the `save_as` method to save the file to a different path.
    /// The todo's are formatted using the `Display` trait and written to the file.
    pub fn save(&self) {
        let mut content = String::new();
        for todo in &self.todos {
            content.push_str(format!("{}\n", todo).as_str());
        }

        std::fs::write(&self.path, content).unwrap();
    }

    /// Saves the `TodoFile` struct to a different file on disk.
    /// Works the same as the `save` method, but you can specify a different path.
    pub fn save_as(&self, path: &str) {
        let mut content = String::new();
        for todo in &self.todos {
            content.push_str(format!("{}\n", todo).as_str());
        }

        std::fs::write(path, content).unwrap();
    }

    /// Changes the status of a todo item.
    /// The index is the index of the todo item in the `todos` vector.
    /// The status is toggled between completed and not completed.
    pub fn change_status(&mut self, index: usize) {
        self.todos[index].toggle_status();
    }

    /// Removes a todo item from the `todos` vector.
    pub fn remove(&mut self, index: usize) {
        self.todos.remove(index);
    }

    /// Adds a todo item to the `todos` vector.
    pub fn add(&mut self, todo: parser::Todo) {
        self.todos.push(todo);
    }

    /// Updates a todo item in the `todos` vector.
    /// Doesn't do anything if the index is out of bounds.
    pub fn update(&mut self, index: usize, todo: parser::Todo) {
        if index >= self.todos.len() {
            return;
        }

        self.todos[index] = todo;
    }

    /// Gets a todo item from the `todos` vector.
    /// Returns None if the index is out of bounds.
    ///
    /// This can be seen as a safe way to access the `todos` vector.
    /// However, the [] operator is also implemented for the `TodoFile` struct.
    pub fn get(&self, index: usize) -> Option<parser::Todo> {
        if index >= self.todos.len() {
            return None;
        }

        Some(self.todos[index].clone())
    }

    /// Returns the number of todo items in the `todos` vector.
    pub fn len(&self) -> usize {
        self.todos.len()
    }

    /// Returns whether the `todos` vector is empty or not.
    pub fn is_empty(&self) -> bool {
        self.todos.is_empty()
    }

    /// Rearranges the todo items into a new vector.
    /// The rearrangement is done by sorting the todo items sequentially in the following
    /// order:
    ///
    /// 1. Not completed todo items sorted by creation date.
    /// 2. Completed todo items sorted by completion date.
    /// 3. Merged list of the two categories.
    ///
    /// This behavior is inspired by the todo.txt cli application.
    pub fn rearrange(&mut self) -> Vec<parser::Todo> {
        // get all the todo's that are not done, sort them by creation date
        let mut not_done = self
            .todos
            .iter()
            .filter(|e| !e.completed)
            .collect::<Vec<&parser::Todo>>();
        not_done.sort_by(|a, b| a.creation.cmp(&b.creation));

        // get all the todo's that are done, sort them by completion date
        let mut done = self
            .todos
            .iter()
            .filter(|e| e.completed)
            .collect::<Vec<&parser::Todo>>();
        done.sort_by(|a, b| a.completion.cmp(&b.completion));

        // merge the two lists
        let mut new_todos = Vec::new();
        new_todos.append(&mut not_done.iter().map(|e| (*e).clone()).collect());
        new_todos.append(&mut done.iter().map(|e| (*e).clone()).collect());

        new_todos
    }

    /// Searches for a query in the todo items.
    /// The query is a string that is searched in the `content` field of the todo items.
    /// This search happens in a case-sensitive manner and takes O(n) time in the worst case.
    pub fn search(&self, query: &str) -> Vec<parser::Todo> {
        self.todos
            .iter()
            .filter(|e| e.content.contains(query))
            .cloned()
            .collect()
    }

    /// Searches for a query in the todo items using a regex.
    /// Regex can be seen as significantly faster than the `search` method.
    /// Alternatively, helper methods like `get_project` and `get_context` can be used to
    /// simplify the search.
    pub fn regex(&self, query: &str) -> Vec<parser::Todo> {
        let re = Regex::new(query).unwrap();
        self.todos
            .iter()
            .filter(|e| re.is_match(&e.content).unwrap())
            .cloned()
            .collect()
    }

    /// Gets all the todo items that have a specific project.
    /// Uses a combination of regex and the `parser::Todo` struct to get the project.
    pub fn get_project(&self, project: &str) -> Vec<parser::Todo> {
        self.todos
            .iter()
            .filter(|e| {
                if let Ok(Some(p)) = e.parse_project() {
                    p == project
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Gets all the todo items that have a specific context.
    /// Works similarly to the `get_project` method.
    pub fn get_context(&self, context: &str) -> Vec<parser::Todo> {
        self.todos
            .iter()
            .filter(|e| {
                if let Ok(Some(c)) = e.parse_context() {
                    c == context
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Lists all of the projects in the todo items in a sorted and deduplicated manner.
    pub fn list_projects(&self) -> Vec<String> {
        let mut projects = Vec::new();
        for todo in &self.todos {
            if let Ok(Some(project)) = todo.parse_project() {
                projects.push(project);
            }
        }

        projects.sort();
        projects.dedup();

        projects
    }

    /// Lists all of the contexts in the todo items in a sorted and deduplicated manner.
    pub fn list_contexts(&self) -> Vec<String> {
        let mut contexts = Vec::new();
        for todo in &self.todos {
            if let Ok(Some(context)) = todo.parse_context() {
                contexts.push(context);
            }
        }

        contexts.sort();
        contexts.dedup();

        contexts
    }

    /// Lists all of the tags in the todo items in a sorted and deduplicated manner.
    pub fn list_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        for todo in &self.todos {
            if let Ok(map) = todo.parse_tags() {
                for (k, _) in map {
                    tags.push(k);
                }
            }
        }

        tags.sort();
        tags.dedup();

        tags
    }

    /// Experimental: Lists all of the hashtags in the todo items in a sorted and deduplicated
    /// manner.
    pub fn list_hashtags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        for todo in &self.todos {
            if let Ok(t) = todo.parse_hashtags() {
                for tag in t {
                    tags.push(tag);
                }
            }
        }

        tags.sort();
        tags.dedup();

        tags
    }

    /// Returns a vector of all the todo items that are completed.
    pub fn completed(&self) -> Vec<parser::Todo> {
        self.todos.iter().filter(|e| e.completed).cloned().collect()
    }

    /// Returns a vector of all the todo items that are not completed.
    pub fn not_completed(&self) -> Vec<parser::Todo> {
        self.todos
            .iter()
            .filter(|e| !e.completed)
            .cloned()
            .collect()
    }

    /// Returns a vector of all the todo items that are due today.
    /// Uses the chrono library to get the current date and compares it with the due date of the
    /// todo item.
    pub fn due_today(&self) -> Vec<parser::Todo> {
        let today = chrono::Local::now().naive_local().date();
        self.todos
            .iter()
            .filter(|e| {
                if let Ok(Some(date)) = e.parse_due() {
                    date == today
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Returns a vector of all the todo items that are due tomorrow.
    /// Similar to the `due_today` method, but the date can be specified.
    pub fn due_on(&self, date: chrono::NaiveDate) -> Vec<parser::Todo> {
        self.todos
            .iter()
            .filter(|e| {
                if let Ok(Some(d)) = e.parse_due() {
                    d == date
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Returns the file as a json of parsed todo items.
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!(self.todos)
    }

    /// Parses a json value and returns a `TodoFile` struct.
    pub fn from_json(path: &Path, json: serde_json::Value) -> Self {
        let todos = json
            .as_array()
            .unwrap()
            .iter()
            .map(|e| serde_json::from_value(e.clone()).unwrap())
            .collect();

        Self {
            path: path.to_path_buf(),
            todos,
            content: format!("{}", json),
        }
    }
}

impl Display for TodoFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, todo) in self.todos.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, todo)?;
        }

        Ok(())
    }
}

impl std::ops::Index<usize> for TodoFile {
    type Output = parser::Todo;

    fn index(&self, index: usize) -> &Self::Output {
        &self.todos[index]
    }
}

impl std::ops::IndexMut<usize> for TodoFile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.todos[index]
    }
}

impl std::iter::IntoIterator for TodoFile {
    type Item = parser::Todo;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.todos.into_iter()
    }
}

#[cfg(test)]
#[test]
fn test_status_toggle() {
    let mut t =
        TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123\n");
    t.change_status(0);
    assert!(!t[0].completed);
}

#[test]
fn test_list_projects() {
    let t = TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123\n (B) 2024-08-02 Nice +hi @wow\n");
    let projects = t.list_projects();
    assert_eq!(projects, vec!["hello", "hi"]);
}

#[test]
fn test_list_contexts() {
    let t = TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123\n (B) 2024-08-02 Nice +hi @wow\n");
    let contexts = t.list_contexts();
    assert_eq!(contexts, vec!["wow"]);
}

#[test]
fn test_rearrange() {
    let mut t = TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123\n (B) 2024-08-02 Nice +hi @wow\n");
    let rearranged = t.rearrange();
    assert_eq!(rearranged[0].title, "Nice");
}

#[test]
fn test_due_on() {
    let t = TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15\n (B) 2024-08-02 Nice +hi @wow due:2021-08-16\n");
    let due_today = t.due_on(chrono::NaiveDate::from_ymd_opt(2021, 8, 15).unwrap());
    assert_eq!(due_today[0].title, "Hello World");
}

#[test]
fn test_search() {
    let t = TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15\n (B) 2024-08-02 Nice +hi @wow due:2021-08-16\n");
    let search = t.search("Hello");
    assert_eq!(search[0].title, "Hello World");
}
