// batch-rename: 批量重命名文件
// 用法: batch-rename <glob模式> --pattern <查找> --replace <替换>
// 示例: batch-rename "*.jpg" --pattern "photo_" --replace "img_"

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 解析参数
    let (glob_pattern, find, replace) = match parse_args(&args) {
        Some(parsed) => parsed,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    // 查找匹配的文件
    let files = find_files(&glob_pattern);
    if files.is_empty() {
        println!("没有找到匹配的文件");
        return;
    }

    // 计算重命名操作
    let renames: Vec<(PathBuf, PathBuf)> = files
        .iter()
        .filter_map(|path| {
            let filename = path.file_name()?.to_str()?;
            if filename.contains(&find) {
                let new_name = filename.replace(&find, &replace);
                let new_path = path.with_file_name(new_name);
                Some((path.clone(), new_path))
            } else {
                None
            }
        })
        .collect();

    if renames.is_empty() {
        println!("没有需要重命名的文件");
        return;
    }

    // 预览
    println!("预览：");
    for (old, new) in &renames {
        println!(
            "  {} -> {}",
            old.file_name().unwrap().to_string_lossy(),
            new.file_name().unwrap().to_string_lossy()
        );
    }
    println!();

    // 确认
    if !common::confirm("确认执行？") {
        println!("已取消");
        return;
    }

    // 执行重命名
    let mut success = 0;
    let mut failed = 0;

    for (old, new) in &renames {
        match fs::rename(old, new) {
            Ok(_) => {
                success += 1;
            }
            Err(e) => {
                eprintln!("重命名失败 {}: {}", old.display(), e);
                failed += 1;
            }
        }
    }

    println!("完成：成功 {} 个，失败 {} 个", success, failed);
}

fn parse_args(args: &[String]) -> Option<(String, String, String)> {
    if args.len() < 6 {
        return None;
    }

    let glob_pattern = args[1].clone();
    let mut find = None;
    let mut replace = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--pattern" if i + 1 < args.len() => {
                find = Some(args[i + 1].clone());
                i += 2;
            }
            "--replace" if i + 1 < args.len() => {
                replace = Some(args[i + 1].clone());
                i += 2;
            }
            _ => i += 1,
        }
    }

    Some((glob_pattern, find?, replace?))
}

fn find_files(pattern: &str) -> Vec<PathBuf> {
    glob::glob(pattern)
        .map(|paths| paths.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

fn print_usage() {
    eprintln!("用法: batch-rename <glob模式> --pattern <查找> --replace <替换>");
    eprintln!("示例: batch-rename \"*.jpg\" --pattern \"photo_\" --replace \"img_\"");
}
