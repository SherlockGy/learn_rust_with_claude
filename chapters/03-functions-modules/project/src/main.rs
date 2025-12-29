mod counter;
mod output;

use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        // 从标准输入读取
        let mut text = String::new();
        io::stdin().read_to_string(&mut text).unwrap();
        let result = counter::count_text(&text);
        output::print_result(&result, None);
    } else {
        // 从文件读取
        for filename in &args {
            match fs::read_to_string(filename) {
                Ok(text) => {
                    let result = counter::count_text(&text);
                    output::print_result(&result, Some(filename));
                }
                Err(e) => {
                    eprintln!("word-count: {}: {}", filename, e);
                }
            }
        }
    }
}
