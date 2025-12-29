//! freq - 词频统计工具

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};

fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        // 清理标点符号并转小写
        let word: String = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();

        if !word.is_empty() {
            *counts.entry(word).or_insert(0) += 1;
        }
    }

    counts
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // 读取文本
    let text = if args.len() > 1 {
        fs::read_to_string(&args[1]).expect("无法读取文件")
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).expect("无法读取输入");
        buf
    };

    // 统计词频
    let counts = count_words(&text);

    // 排序并输出
    let mut items: Vec<_> = counts.iter().collect();
    items.sort_by(|a, b| b.1.cmp(a.1));

    // 获取 --top 参数
    let top_n = args.iter()
        .position(|a| a == "--top")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    println!("{:15} {:>8}", "单词", "次数");
    println!("{}", "-".repeat(25));

    for (word, count) in items.iter().take(top_n) {
        println!("{:15} {:>8}", word, count);
    }

    println!("\n总计: {} 个不同单词", counts.len());
}
