use std::fs::read_to_string;
fn main() {
    let content = read_to_string("testfile").unwrap();
    println!("{}", content);

}
