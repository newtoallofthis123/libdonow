//! # libdonow
//!
//! A rust library for the [todo.txt](https://github.com/todotxt/todo.txt) format.
//! The library supports all of the features of format, including priorities, contexts, projects, and due dates.
//! This library is the basis for the [donow](https://github.com/newtoallofthis123/donow) application
//! which is a native todo.txt application written using the Tauri Framework.
//!
//! libdonow is has various features that make it easy to work with todo.txt files.
//! You can simply open up a todo.txt file using the custom `TodoFile` wrapper struct that has
//! various methods to interact with the file.
//!
//! ```rust
//! use libdonow::file::TodoFile;
//! let mut file = TodoFile::from_string("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow
//! due:123\n");
//! file.rearrange();
//! println!("{}", file);
//! ```
//! The TodoFile struct also has various implementations and methods to feel like a Vec<Todo> struct, but with some extra features.
//!
//! The library also has powerful features to work with a single todo item.
//! Each todo item is parsed using some fancy regex features and is stored in a struct called `Todo`.
//! the `Todo` struct follows a only what's needed approach so you have various functions and utilities to retrieve only what is necessary
//! without having to parse the entire todo item.
pub mod file;
pub mod parser;
