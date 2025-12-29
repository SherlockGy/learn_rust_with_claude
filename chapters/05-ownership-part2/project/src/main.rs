use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    let count_mode = args.len() > 1 && args[1] == "-c";

    let stdin = io::stdin();
    let mut prev_line = String::new();
    let mut count: usize = 0;
    let mut first = true;

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if first {
            prev_line = line;
            count = 1;
            first = false;
        } else if line == prev_line {
            count += 1;
        } else {
            print_line(&prev_line, count, count_mode);
            prev_line = line;
            count = 1;
        }
    }

    // 输出最后一组
    if !first {
        print_line(&prev_line, count, count_mode);
    }
}

fn print_line(line: &str, count: usize, count_mode: bool) {
    if count_mode {
        println!("{:>7} {}", count, line);
    } else {
        println!("{}", line);
    }
}
