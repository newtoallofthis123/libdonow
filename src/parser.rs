//! # Todo Item
//!
//! A struct representing a single todo item.
//! The struct has various fields that represent the various parts of a todo item.
//! These follow the todo.txt format correctly and any and all additions provided by the libray are independent of the format.
//! The struct is highly optimize for being as fast as possible and only parsing what is needed.
//! It follows a only what's needed approach and provides various functions to retrieve only the
//! necessary parts of the todo item without having to fill in all the fields.
//!
//! So if you only need the project of a todo item, you can simply call the `parse_project`
//! function and it will return the project of the todo item without even knowing the other fields.
//!
//! The struct parses using `fancy_regex` which is a regex library that is highly optimized for
//! speed and performance. All of the parsing is done using regex and hence is very fast and also
//! intuitively easy to understand.
//!
//! The struct also implements the `Display` trait which ensures that the struct can be printed
//! as found in the todo.txt file with all of the changes that have been made to the todo item.
//!
//! ```rust
//! use libdonow::parser::Todo;
//!
//! let mut t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123");
//! t.fill().unwrap();
//! println!("{}", t.parse_project().unwrap().unwrap());
//! println!("{}", t.parse_context().unwrap().unwrap());
//! ```

use std::{fmt::Display, str::FromStr};

use fancy_regex::Regex;
use hashbrown::HashMap;

/// A struct representing a single todo item.
/// A new todo item can be created using the `new` function which takes a string slice as an
/// argument. The string slice is the content of the todo item.
/// This function however doesn't parse the todo item, it just stores the content.
/// You can choose to either parse the complete todo item using the `parse` function or
/// simply fill in the missing fields using the `fill` function.
///
/// However, a more powerful way to interact with the todo item would be using functions like
/// `parse_project`, `parse_context`, `parse_tags`, `parse_priority` which reduce complexity
/// by only returning relevant parts of the todo item.
///
/// ```rust
/// use libdonow::parser::Todo;
///
/// let mut t = Todo::parse("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123").unwrap();
/// println!("{}", t.parse_project().unwrap().unwrap());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Todo {
    /// The title of the todo item.
    pub title: String,
    /// The status of the todo item.
    pub completed: bool,
    /// The priority of the todo item.
    pub priority: Option<String>,
    /// The completion date of the todo item.
    pub completion: Option<chrono::NaiveDate>,
    /// The creation date of the todo item.
    pub creation: Option<chrono::NaiveDate>,
    /// The project of the todo item.
    pub project: Option<String>,
    /// The context of the todo item.
    pub context: Option<String>,
    /// The tags of the todo item.
    pub others: HashMap<String, String>,
    /// The content of the todo item.
    pub content: String,
}

impl Todo {
    /// Parses a todo item from a string slice.
    /// The function takes a string slice as an argument and returns a `Result` with the `Todo`
    /// struct
    /// It can return an error if the todo item is not in the correct format or if the regex
    /// parsing fails.
    ///
    /// This essentially calls all of the parsing functions and fills in the fields of the todo
    /// item.
    /// This is not needed if you only need a part of the todo item. In that case, you can use
    /// the other parsing functions.
    pub fn parse(s: &str) -> Result<Self, TodoErr> {
        let mut t = Todo::new(s);
        t.completed = t.content.starts_with('x');
        if t.completed {
            t.content = t.content[1..].trim().to_string();
        }
        t.project = t.parse_project()?;
        t.context = t.parse_context()?;
        t.others = t.parse_tags()?;
        t.priority = t.parse_priority()?;
        t.title = t.parse_title()?;

        let dates = t.parse_dates()?;
        t.creation = dates.0;
        t.completion = dates.1;

        Ok(t)
    }

    /// Fills in the missing fields of the todo item.
    /// It uses the `parse` function to parse the todo item and then replaces the current todo
    /// item with the parsed todo item.
    pub fn fill(&mut self) -> Result<(), TodoErr> {
        let t = Todo::parse(&self.content)?;
        *self = t;

        Ok(())
    }

    /// Smart Parse builds upon the `parse` function and fills in the missing fields with
    /// default values that are determined by the library when the todo item is not in the
    /// correct format.
    /// It is still expermental and may not work as expected.
    pub fn smart_parse(s: &str) -> Result<Self, TodoErr> {
        let mut t = Todo::parse(s)?;

        if t.creation.is_none() {
            t.creation = Some(chrono::Local::now().naive_local().date());
        }

        if t.priority.is_none() {
            t.priority = Some("D".to_string());
        }

        Ok(t)
    }

    /// Creates a new todo item with the content filled in.
    /// All of the values other than the content are set to default values.
    /// Use the `fill` function to fill in the missing fields or the `parse` function to create
    /// a todo item with all of the fields filled in.
    pub fn new(s: &str) -> Self {
        Todo {
            title: String::new(),
            completed: false,
            priority: None,
            completion: None,
            creation: None,
            project: None,
            context: None,
            others: HashMap::new(),
            content: s.to_string(),
        }
    }

    /// Parses the due date of the todo item.
    /// This function returns an `Option` with the `NaiveDate` of the due date.
    pub fn parse_due(&self) -> Result<Option<chrono::NaiveDate>, TodoErr> {
        let date_re =
            Regex::new("due:(\\d{4}-\\d{2}-\\d{2})").map_err(|_| TodoErr::RegexParseErr)?;
        match date_re.find(&self.content) {
            Ok(s) => Ok(s.map(|p| {
                chrono::NaiveDate::parse_from_str(&p.as_str()[4..], "%Y-%m-%d")
                    .expect("Failed to parse date")
            })),
            Err(_) => Err(TodoErr::RegexParseErr),
        }
    }

    /// Parses the project of the todo item.
    /// If there are two projects in the todo item, it returns the first one
    /// A project is in the format `+project`
    pub fn parse_project(&self) -> Result<Option<String>, TodoErr> {
        let project_re = Regex::new("\\+(\\w+)").map_err(|_| TodoErr::RegexParseErr)?;
        match project_re.find(&self.content) {
            Ok(s) => Ok(s.map(|p| p.as_str()[1..].to_string())),
            Err(_) => Err(TodoErr::RegexParseErr),
        }
    }

    /// Parses the context of the todo item.
    /// If there are two contexts in the todo item, it returns the first one
    /// A context is in the format `@context`
    pub fn parse_context(&self) -> Result<Option<String>, TodoErr> {
        let context_re = Regex::new("\\@(\\w+)").map_err(|_| TodoErr::RegexParseErr)?;
        match context_re.find(&self.content) {
            Ok(s) => Ok(s.map(|p| p.as_str()[1..].to_string())),
            Err(_) => Err(TodoErr::RegexParseErr),
        }
    }

    /// Parses the tags of the todo item.
    /// Tags are in the format `key:value` and are separated by a space.
    pub fn parse_tags(&self) -> Result<HashMap<String, String>, TodoErr> {
        let tags_re = Regex::new("(\\w+):(\\S+)").map_err(|_| TodoErr::RegexParseErr)?;
        let mut map = HashMap::new();

        let iter = tags_re.find_iter(&self.content);
        iter.for_each(|e| {
            if let Ok(e) = e {
                let split = e.as_str().to_string();
                if split.split_once(':').is_some() {
                    let (k, v) = split.split_once(':').unwrap();
                    map.insert(k.to_string(), v.to_string());
                }
            }
        });

        Ok(map)
    }

    /// Parses the priority of the todo item.
    /// A priority is in the format `(A)` and is at the start of the todo item.
    pub fn parse_priority(&self) -> Result<Option<String>, TodoErr> {
        let p_re = Regex::new("\\((\\w+)\\)").map_err(|_| TodoErr::RegexParseErr)?;
        match p_re.find(&self.content) {
            Ok(s) => Ok(s.map(|p| {
                p.as_str()
                    .strip_prefix('(')
                    .unwrap()
                    .strip_suffix(')')
                    .unwrap()
                    .to_string()
            })),
            Err(_) => Err(TodoErr::RegexParseErr),
        }
    }

    /// Experimental: Parses the hashtags of the todo item.
    /// Hashtags are in the format `#hashtag` and are separated by a space.
    ///
    /// These are not supported by the todo.txt format and are an experimental feature.
    pub fn parse_hashtags(&self) -> Result<Vec<String>, TodoErr> {
        let tags_re = Regex::new("#(\\w+)").map_err(|_| TodoErr::RegexParseErr)?;
        let mut tags = Vec::new();

        let iter = tags_re.find_iter(&self.content);
        iter.for_each(|e| {
            if let Ok(e) = e {
                tags.push(e.as_str().to_string());
            }
        });

        Ok(tags)
    }

    /// Parses the title of the todo item.
    /// It is guaranteed that the title will be returned and if there is no title, it will return
    /// an error.
    pub fn parse_title(&self) -> Result<String, TodoErr> {
        let mut content = self.content.clone();
        if content.starts_with('x') {
            content = content[1..].trim().to_string();
        }

        // remove anything that starts with a +, @ or () or some:word or a date with - or :
        let re =
            Regex::new("(\\+(\\w+)|\\@(\\w+)|\\((\\w+)\\)|\\w+:(\\S+)|(?<!:)\\d{4}-\\d{2}-\\d{2})")
                .map_err(|_| TodoErr::RegexParseErr)?;
        let title = re.replace_all(&content, "");

        if title.trim().is_empty() {
            Err(TodoErr::NoTitle)
        } else {
            Ok(title.trim().to_string())
        }
    }

    /// Parses the dates of the todo item.
    /// The function returns a tuple with the creation date and the completion date.
    pub fn parse_dates(
        &self,
    ) -> Result<(Option<chrono::NaiveDate>, Option<chrono::NaiveDate>), TodoErr> {
        let date_re =
            Regex::new("(?<!:)(\\d{4}-\\d{2}-\\d{2})").map_err(|_| TodoErr::RegexParseErr)?;
        let mut dates = date_re.find_iter(&self.content);

        let mut creation = None;
        let mut completion = None;

        if let Some(date) = dates.next() {
            creation = Some(
                chrono::NaiveDate::parse_from_str(date.unwrap().as_str(), "%Y-%m-%d")
                    .expect("Failed to parse date"),
            );
        }

        if let Some(date) = dates.next() {
            completion = Some(
                chrono::NaiveDate::parse_from_str(date.unwrap().as_str(), "%Y-%m-%d")
                    .expect("Failed to parse date"),
            );
        }

        Ok((creation, completion))
    }

    /// Toggles the status of the todo item.
    pub fn toggle_status(&mut self) {
        self.completed = !self.completed;
    }

    /// Pretty prints the todo item.
    /// use format! to print the todo item in the todo.txt format.
    pub fn print(&self) {
        println!("Title: {}", self.title);
        if self.completed {
            println!("Completed: Yes");
        } else {
            println!("Completed: No");
        }
        if let Some(p) = &self.priority {
            println!("Priority: {}", p);
        }
        if let Some(p) = &self.project {
            println!("Project: {}", p);
        }
        if let Some(p) = &self.context {
            println!("Context: {}", p);
        }
        if let Some(p) = &self.creation {
            println!("Creation: {}", p);
        }
        if let Some(p) = &self.completion {
            println!("Completion: {}", p);
        }
        if !self.others.is_empty() {
            for (k, v) in &self.others {
                println!("{}: {}", k, v);
            }
        }
    }
}

/// An enum representing the various errors that can occur while parsing a todo item.
#[derive(Debug)]
pub enum TodoErr {
    /// An error that occurs when the todo item has no title.
    NoTitle,
    /// An error that occurs when the regex parsing fails.
    RegexParseErr,
}

impl FromStr for Todo {
    type Err = TodoErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Todo::parse(s)
    }
}

impl Default for Todo {
    fn default() -> Self {
        Todo {
            title: String::new(),
            completed: false,
            priority: None,
            completion: None,
            creation: None,
            project: None,
            context: None,
            others: HashMap::new(),
            content: String::new(),
        }
    }
}

impl Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.completed {
            s.push('x');
        }
        if let Some(p) = &self.priority {
            s.push_str(&format!(" ({})", p));
        }
        if let Some(p) = &self.completion {
            s.push_str(&format!(" {}", p));
        }
        if let Some(p) = &self.creation {
            s.push_str(&format!(" {}", p));
        }
        s.push_str(&format!(" {}", self.title));
        if let Some(p) = &self.project {
            s.push_str(&format!(" +{}", p));
        }
        if let Some(p) = &self.context {
            s.push_str(&format!(" @{}", p));
        }
        for (k, v) in &self.others {
            s.push_str(&format!(" {}:{}", k, v));
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
#[test]
fn test_project_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123");
    assert_eq!(t.parse_project().unwrap(), Some("hello".to_string()));
}

#[test]
fn test_context_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123");
    assert_eq!(t.parse_context().unwrap(), Some("wow".to_string()));
}

#[test]
fn test_tags_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123 some:word");
    let mut map = HashMap::new();
    map.insert("some".to_string(), "word".to_string());
    map.insert("due".to_string(), "123".to_string());
    assert_eq!(t.parse_tags().unwrap(), map);
}

#[test]
fn test_priority_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123 some:word");
    assert_eq!(t.parse_priority().unwrap(), Some("A".to_string()));
}

#[test]
fn test_title_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123 some:word");
    assert_eq!(t.parse_title().unwrap(), "Hello World".to_string());
}

#[test]
fn test_dates_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123 some:word");
    let dates = t.parse_dates().unwrap();
    assert_eq!(
        dates.0.unwrap(),
        chrono::NaiveDate::from_ymd_opt(2024, 8, 15).unwrap()
    );
    assert_eq!(
        dates.1.unwrap(),
        chrono::NaiveDate::from_ymd_opt(2024, 9, 20).unwrap()
    );
}

#[test]
fn test_due_parse() {
    let t =
        Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15 some:word");
    assert_eq!(
        t.parse_due().unwrap().unwrap(),
        chrono::NaiveDate::from_ymd_opt(2021, 8, 15).unwrap()
    );
}

#[test]
fn test_hashtags_parse() {
    let t = Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15 some:word #hello #world");
    assert_eq!(
        t.parse_hashtags().unwrap(),
        vec!["#hello".to_string(), "#world".to_string()]
    );
}

#[test]
fn test_smart_parse() {
    let t = Todo::smart_parse(
        "x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15 some:word",
    )
    .unwrap();
    assert_eq!(
        t.creation.unwrap(),
        chrono::NaiveDate::from_ymd_opt(2024, 8, 15).unwrap()
    );
    assert_eq!(t.priority.unwrap(), "A".to_string());
}

#[test]
fn test_toggle_status() {
    let mut t =
        Todo::parse("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15 some:word")
            .unwrap();
    t.toggle_status();
    assert!(!t.completed);
}

#[test]
fn test_display() {
    let mut t =
        Todo::new("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:2021-08-15 some:word");
    match t.fill() {
        Ok(_) => {}
        Err(e) => println!("{:?}", e),
    }
    assert_eq!(
        t.to_string(),
        "x (A) 2024-09-20 2024-08-15 Hello World +hello @wow due:2021-08-15 some:word"
    );
}
