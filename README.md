# libdonow

A rust library for the [todo.txt](https://github.com/todotxt/todo.txt) format.
The library supports all of the features of format, including priorities, contexts, projects, and due dates.
This library is the basis for the [donow](https://github.com/newtoallofthis123/donow) application
which is a native todo.txt application written using the Tauri Framework.

libdonow is has various features that make it easy to work with todo.txt files.
You can simply open up a todo.txt file using the custom `TodoFile` wrapper struct that has 
various methods to interact with the file.

```rust
use libdonow::file::TodoFile;

fn main(){
    let file = TodoFile::new("todo.txt");
    file.rearrange();
    file.save();
    println!("{}", file);

    let todo = file[0];
    todo.toggle_status();
    println!("{}", todo);
}
```
The TodoFile struct also has various implementations and methods to feel like a Vec<Todo> struct, but with some extra features.

The library also has powerful features to work with a single todo item.
Each todo item is parsed using some fancy regex features and is stored in a struct called `Todo`.
the `Todo` struct follows a only what's needed approach so you have various functions and utilities to retrieve only what is necessary
without having to parse the entire todo item.

For example, the `get_projects` method in the TodoFile struct returns a Vec<String> of all the projects in the todo.txt file.
To do this, it only parses the projects in the todo.txt file and returns them.

```rust
for todo in &self.todos {
    if let Ok(Some(project)) = todo.parse_project() {
        projects.push(project);
    }
}
```

More information about the library can be found in the [docs](docs.rs/libdonow).

## Contributing

If you would like to contribute to the project, feel free to fork the repository and submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
