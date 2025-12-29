use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut prev_line = String::new();
    let mut first = true;

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if first || line != prev_line {
            println!("{}", line);
            prev_line = line;
            first = false;
        }
    }
}
