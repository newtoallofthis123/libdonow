use libdonow::parser::Todo;

fn main() {
    match Todo::parse("x (A) 2024-08-15 2024-09-20 Hello World +hello @wow due:123") {
        Ok(e) => {
            println!("{}", e);
        }
        Err(e) => println!("{:?}", e),
    }
}
