use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line_count: usize = 0;
    let mut word_count: usize = 0;
    let mut char_count: usize = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        line_count += 1;
        word_count += line.split_whitespace().count();
        char_count += line.chars().count() + 1; // +1 补回换行符
    }

    // 处理边界情况：假设最后一行没有换行符
    if line_count > 0 {
        char_count -= 1; // 减去最后一行多加的换行符
    }

    println!("{:>8}{:>8}{:>8}", line_count, word_count, char_count);
}
