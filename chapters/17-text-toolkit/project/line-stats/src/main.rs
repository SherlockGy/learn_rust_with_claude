// line-stats: 代码行统计工具
// 用法: line-stats <文件或glob模式>...
// 示例: line-stats src/**/*.rs

use common::FileStats;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("用法: line-stats <文件或glob模式>...");
        eprintln!("示例: line-stats src/**/*.rs");
        std::process::exit(1);
    }

    // 展开所有 glob 模式
    let files: Vec<PathBuf> = args
        .iter()
        .flat_map(|pattern| {
            glob::glob(pattern)
                .map(|paths| paths.filter_map(Result::ok).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .filter(|p| p.is_file())
        .collect();

    if files.is_empty() {
        println!("没有找到匹配的文件");
        return;
    }

    // 打印表头
    println!(
        "{:<40} {:>8} {:>8} {:>8}",
        "文件", "行数", "空行", "代码行"
    );
    println!("{}", "-".repeat(68));

    // 统计每个文件
    let mut total = FileStats::default();

    for path in &files {
        match common::stats_file(path) {
            Ok(stats) => {
                // 截断过长的文件名
                let display_name = path.to_string_lossy();
                let display_name = if display_name.len() > 38 {
                    format!("...{}", &display_name[display_name.len() - 35..])
                } else {
                    display_name.to_string()
                };

                println!(
                    "{:<40} {:>8} {:>8} {:>8}",
                    display_name, stats.lines, stats.blank, stats.code
                );

                total.lines += stats.lines;
                total.blank += stats.blank;
                total.code += stats.code;
                total.bytes += stats.bytes;
            }
            Err(e) => {
                eprintln!("无法读取 {}: {}", path.display(), e);
            }
        }
    }

    // 打印总计
    println!("{}", "-".repeat(68));
    println!(
        "{:<40} {:>8} {:>8} {:>8}",
        format!("总计 ({} 个文件)", files.len()),
        total.lines,
        total.blank,
        total.code
    );
    println!("总字节数: {} bytes", total.bytes);
}
