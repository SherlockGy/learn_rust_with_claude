# 第 17 章：实用文本处理工具集

## 本章目标

学完本章，你将能够：
- 使用 Workspace 组织多个相关工具
- 实现批量文件操作的安全模式
- 流式处理大文件
- 构建 text-toolkit 工具集

---

## 前置知识

- 第 13 章：Cargo Workspace
- 第 16 章：文件与 I/O

---

## 项目：text-toolkit 工具集

### 功能概览

一个包含多个文本处理工具的 Workspace：

```
text-toolkit/
├── Cargo.toml          # workspace 配置
├── common/             # 共享库
│   ├── Cargo.toml
│   └── src/lib.rs
├── batch-rename/       # 批量重命名工具
│   ├── Cargo.toml
│   └── src/main.rs
└── line-stats/         # 代码行统计工具
    ├── Cargo.toml
    └── src/main.rs
```

### 各工具功能

```bash
# batch-rename：批量重命名文件
$ batch-rename "*.jpg" --find "photo_" --replace "img_"
预览：
  photo_001.jpg -> img_001.jpg
  photo_002.jpg -> img_002.jpg
确认执行？(y/N) y
✓ 已重命名 2 个文件

# line-stats：代码行统计
$ line-stats src/
文件                    行数    空行    代码行   注释行
src/main.rs             156     23      120      13
src/lib.rs              89      12      70       7
src/utils/mod.rs        45      8       35       2
────────────────────────────────────────────────────
总计                    290     43      225      22
```

---

## Workspace 配置

### 根目录 Cargo.toml

```toml
[workspace]
members = [
    "common",
    "batch-rename",
    "line-stats",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name"]

[workspace.dependencies]
clap = { version = "4", features = ["derive"] }
walkdir = "2"
regex = "1"
```

### common/Cargo.toml

```toml
[package]
name = "common"
version.workspace = true
edition.workspace = true

[dependencies]
```

### batch-rename/Cargo.toml

```toml
[package]
name = "batch-rename"
version.workspace = true
edition.workspace = true

[dependencies]
common = { path = "../common" }
clap.workspace = true
regex.workspace = true
walkdir.workspace = true
```

---

## common 共享库

提供各工具共用的功能：

```rust
// common/src/lib.rs

use std::io::{self, Write};
use std::path::Path;

/// 请求用户确认
pub fn confirm(prompt: &str) -> bool {
    print!("{} (y/N) ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 检查路径是否应该被忽略
pub fn should_ignore(path: &Path, ignore_patterns: &[&str]) -> bool {
    for pattern in ignore_patterns {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == *pattern || name.starts_with(pattern) {
                return true;
            }
        }
    }
    false
}

/// 默认忽略的目录
pub const DEFAULT_IGNORE: &[&str] = &[".git", "target", "node_modules", ".idea", "__pycache__"];
```

---

## batch-rename 实现

### 核心功能

批量重命名工具需要考虑：
1. **预览模式**：先展示将要进行的更改
2. **用户确认**：防止误操作
3. **原子操作**：要么全部成功，要么全部失败
4. **错误恢复**：出错时回滚已完成的操作

### 完整代码

```rust
// batch-rename/src/main.rs

use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "batch-rename")]
#[command(about = "批量重命名文件")]
struct Args {
    /// 文件模式（如 *.jpg）
    pattern: String,

    /// 要查找的字符串
    #[arg(short, long)]
    find: String,

    /// 替换为的字符串
    #[arg(short, long)]
    replace: String,

    /// 搜索目录（默认当前目录）
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,

    /// 使用正则表达式
    #[arg(long)]
    regex: bool,

    /// 递归处理子目录
    #[arg(short = 'R', long)]
    recursive: bool,

    /// 跳过确认直接执行
    #[arg(short = 'y', long)]
    yes: bool,

    /// 只预览，不实际执行
    #[arg(long)]
    dry_run: bool,
}

struct RenameOperation {
    from: PathBuf,
    to: PathBuf,
}

fn main() {
    let args = Args::parse();

    // 收集要重命名的文件
    let operations = collect_operations(&args);

    if operations.is_empty() {
        println!("没有找到匹配的文件");
        return;
    }

    // 显示预览
    println!("将要执行的重命名操作：\n");
    for op in &operations {
        println!("  {} -> {}",
            op.from.display(),
            op.to.file_name().unwrap().to_string_lossy()
        );
    }
    println!("\n共 {} 个文件", operations.len());

    // 如果是 dry-run 模式，到此为止
    if args.dry_run {
        println!("\n(dry-run 模式，不会实际执行)");
        return;
    }

    // 确认执行
    if !args.yes && !common::confirm("\n确认执行？") {
        println!("已取消");
        return;
    }

    // 执行重命名
    let result = execute_renames(&operations);

    match result {
        Ok(count) => println!("\n✓ 成功重命名 {} 个文件", count),
        Err(e) => eprintln!("\n✗ 错误: {}", e),
    }
}

fn collect_operations(args: &Args) -> Vec<RenameOperation> {
    let mut operations = Vec::new();

    let walker = if args.recursive {
        WalkDir::new(&args.dir)
    } else {
        WalkDir::new(&args.dir).max_depth(1)
    };

    let regex = if args.regex {
        Some(Regex::new(&args.find).expect("无效的正则表达式"))
    } else {
        None
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // 检查是否匹配模式
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if !matches_pattern(file_name, &args.pattern) {
            continue;
        }

        // 计算新文件名
        let new_name = if let Some(ref re) = regex {
            re.replace_all(file_name, args.replace.as_str()).to_string()
        } else {
            file_name.replace(&args.find, &args.replace)
        };

        // 如果文件名没有变化，跳过
        if new_name == file_name {
            continue;
        }

        let new_path = path.with_file_name(&new_name);

        operations.push(RenameOperation {
            from: path.to_path_buf(),
            to: new_path,
        });
    }

    operations
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return name.ends_with(&format!(".{}", ext));
    }

    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }

    name == pattern
}

fn execute_renames(operations: &[RenameOperation]) -> Result<usize, String> {
    let mut completed = Vec::new();

    for op in operations {
        // 检查目标是否已存在
        if op.to.exists() {
            // 回滚已完成的操作
            rollback(&completed);
            return Err(format!("目标文件已存在: {}", op.to.display()));
        }

        // 执行重命名
        if let Err(e) = fs::rename(&op.from, &op.to) {
            rollback(&completed);
            return Err(format!("重命名失败 {}: {}", op.from.display(), e));
        }

        completed.push(RenameOperation {
            from: op.to.clone(),
            to: op.from.clone(),
        });
    }

    Ok(completed.len())
}

fn rollback(completed: &[RenameOperation]) {
    eprintln!("\n正在回滚...");
    for op in completed.iter().rev() {
        if let Err(e) = fs::rename(&op.from, &op.to) {
            eprintln!("回滚失败 {}: {}", op.from.display(), e);
        }
    }
}
```

---

## line-stats 实现

### 核心功能

代码行统计工具需要：
1. **流式处理**：支持大文件
2. **分类统计**：空行、代码行、注释行
3. **忽略目录**：跳过 .git、target 等
4. **多种输出格式**：表格、JSON

### 完整代码

```rust
// line-stats/src/main.rs

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "line-stats")]
#[command(about = "统计代码行数")]
struct Args {
    /// 要统计的目录或文件
    #[arg(default_value = ".")]
    path: PathBuf,

    /// 要统计的文件扩展名（逗号分隔）
    #[arg(short, long, default_value = "rs,toml,md")]
    extensions: String,

    /// 输出 JSON 格式
    #[arg(long)]
    json: bool,

    /// 忽略的目录（逗号分隔）
    #[arg(long, default_value = ".git,target,node_modules")]
    ignore: String,
}

#[derive(Default, Clone)]
struct FileStats {
    path: String,
    total_lines: usize,
    blank_lines: usize,
    code_lines: usize,
    comment_lines: usize,
}

impl FileStats {
    fn add(&mut self, other: &FileStats) {
        self.total_lines += other.total_lines;
        self.blank_lines += other.blank_lines;
        self.code_lines += other.code_lines;
        self.comment_lines += other.comment_lines;
    }
}

fn main() {
    let args = Args::parse();

    let extensions: Vec<&str> = args.extensions.split(',').collect();
    let ignore_dirs: Vec<&str> = args.ignore.split(',').collect();

    let mut all_stats = Vec::new();
    let mut total = FileStats::default();
    total.path = "总计".to_string();

    // 遍历文件
    for entry in WalkDir::new(&args.path)
        .into_iter()
        .filter_entry(|e| !should_ignore(e.path(), &ignore_dirs))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // 检查扩展名
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !extensions.contains(&ext) {
            continue;
        }

        // 统计文件
        if let Ok(stats) = analyze_file(path) {
            total.add(&stats);
            all_stats.push(stats);
        }
    }

    // 输出结果
    if args.json {
        print_json(&all_stats, &total);
    } else {
        print_table(&all_stats, &total);
    }
}

fn should_ignore(path: &Path, ignore_dirs: &[&str]) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| ignore_dirs.contains(&name))
        .unwrap_or(false)
}

fn analyze_file(path: &Path) -> std::io::Result<FileStats> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut stats = FileStats {
        path: path.display().to_string(),
        ..Default::default()
    };

    let mut in_block_comment = false;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        stats.total_lines += 1;

        if trimmed.is_empty() {
            stats.blank_lines += 1;
            continue;
        }

        // 简单的注释检测（针对 Rust）
        if in_block_comment {
            stats.comment_lines += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            in_block_comment = true;
            stats.comment_lines += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            stats.comment_lines += 1;
            continue;
        }

        stats.code_lines += 1;
    }

    Ok(stats)
}

fn print_table(stats: &[FileStats], total: &FileStats) {
    // 表头
    println!("{:<40} {:>8} {:>8} {:>8} {:>8}",
        "文件", "行数", "空行", "代码行", "注释行"
    );
    println!("{}", "─".repeat(76));

    // 文件统计
    for s in stats {
        let short_path = if s.path.len() > 38 {
            format!("...{}", &s.path[s.path.len()-35..])
        } else {
            s.path.clone()
        };

        println!("{:<40} {:>8} {:>8} {:>8} {:>8}",
            short_path, s.total_lines, s.blank_lines, s.code_lines, s.comment_lines
        );
    }

    // 总计
    println!("{}", "─".repeat(76));
    println!("{:<40} {:>8} {:>8} {:>8} {:>8}",
        total.path, total.total_lines, total.blank_lines,
        total.code_lines, total.comment_lines
    );
}

fn print_json(stats: &[FileStats], total: &FileStats) {
    println!("{{");
    println!("  \"files\": [");

    for (i, s) in stats.iter().enumerate() {
        let comma = if i < stats.len() - 1 { "," } else { "" };
        println!("    {{");
        println!("      \"path\": \"{}\",", s.path.replace('\\', "\\\\"));
        println!("      \"total_lines\": {},", s.total_lines);
        println!("      \"blank_lines\": {},", s.blank_lines);
        println!("      \"code_lines\": {},", s.code_lines);
        println!("      \"comment_lines\": {}", s.comment_lines);
        println!("    }}{}", comma);
    }

    println!("  ],");
    println!("  \"total\": {{");
    println!("    \"total_lines\": {},", total.total_lines);
    println!("    \"blank_lines\": {},", total.blank_lines);
    println!("    \"code_lines\": {},", total.code_lines);
    println!("    \"comment_lines\": {}", total.comment_lines);
    println!("  }}");
    println!("}}");
}
```

---

## 流式处理大文件

对于大文件，必须使用流式处理，避免将整个文件加载到内存：

```rust
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

/// 流式处理大文件，逐行转换
fn process_large_file(
    input: &Path,
    output: &Path,
    transform: impl Fn(&str) -> String,
) -> std::io::Result<u64> {
    let reader = BufReader::new(File::open(input)?);
    let mut writer = std::io::BufWriter::new(File::create(output)?);

    let mut lines_processed = 0u64;

    for line in reader.lines() {
        let line = line?;
        let transformed = transform(&line);
        writeln!(writer, "{}", transformed)?;
        lines_processed += 1;

        // 定期刷新，避免缓冲区过大
        if lines_processed % 10000 == 0 {
            writer.flush()?;
        }
    }

    writer.flush()?;
    Ok(lines_processed)
}

// 使用示例：将文件中所有文本转为大写
fn to_uppercase_file(input: &Path, output: &Path) -> std::io::Result<u64> {
    process_large_file(input, output, |line| line.to_uppercase())
}
```

---

## 原子操作模式

批量操作时，确保要么全部成功，要么全部失败：

```rust
use std::fs;
use std::path::Path;

/// 批量文件操作的事务模式
struct FileTransaction {
    operations: Vec<FileOperation>,
    completed: Vec<FileOperation>,
}

enum FileOperation {
    Rename { from: PathBuf, to: PathBuf },
    Copy { from: PathBuf, to: PathBuf },
    Delete { path: PathBuf, backup: Option<Vec<u8>> },
}

impl FileTransaction {
    fn new() -> Self {
        FileTransaction {
            operations: Vec::new(),
            completed: Vec::new(),
        }
    }

    fn add_rename(&mut self, from: PathBuf, to: PathBuf) {
        self.operations.push(FileOperation::Rename { from, to });
    }

    fn execute(&mut self) -> Result<(), String> {
        for op in &self.operations {
            match op {
                FileOperation::Rename { from, to } => {
                    fs::rename(from, to).map_err(|e| e.to_string())?;
                    self.completed.push(FileOperation::Rename {
                        from: to.clone(),
                        to: from.clone(),
                    });
                }
                // 其他操作...
                _ => {}
            }
        }
        Ok(())
    }

    fn rollback(&self) {
        for op in self.completed.iter().rev() {
            match op {
                FileOperation::Rename { from, to } => {
                    let _ = fs::rename(from, to);
                }
                _ => {}
            }
        }
    }
}
```

---

## 最佳实践

### 批量文件操作

| 场景 | 推荐做法 | 原因 |
|------|---------|------|
| 危险操作 | 先预览，用户确认 | 防止误操作 |
| 多文件操作 | 事务模式，支持回滚 | 保证一致性 |
| 大量文件 | 显示进度 | 用户体验 |
| 失败处理 | 记录失败文件，继续其他 | 最大化成功率 |

### 文本处理

| 场景 | 推荐做法 | 原因 |
|------|---------|------|
| 大文件 | 流式处理 | 避免内存爆炸 |
| 文件修改 | 原子写入 | 避免数据损坏 |
| 编码处理 | 检测或指定编码 | 避免乱码 |
| 换行符 | 统一处理 CRLF/LF | 跨平台兼容 |

### 常见陷阱

| 陷阱 | 问题 | 解决方案 |
|------|------|---------|
| 内存爆炸 | 大文件全部加载 | BufReader 流式读取 |
| 数据丢失 | 写入过程中断 | 原子写入（临时文件 + 重命名） |
| 权限问题 | 无法读写某些文件 | 检查权限，优雅跳过 |
| 路径编码 | 非 UTF-8 文件名 | 使用 OsStr |

---

## 要点回顾

1. **Workspace** 组织相关工具
2. **共享库** 复用通用功能
3. **预览模式** 防止误操作
4. **流式处理** 支持大文件
5. **原子操作** 保证数据安全
6. **事务回滚** 保证一致性

---

## 练习

### 练习 1：忽略目录增强

为 line-stats 添加从 .gitignore 文件读取忽略规则的功能。

### 练习 2：dup-finder

实现一个查找重复文件的工具：
- 按文件大小预筛选
- 用 MD5/SHA256 确认重复
- 输出重复文件组

### 练习 3：正则替换

为 batch-rename 添加捕获组替换功能：
```bash
$ batch-rename "*.jpg" --regex "photo_(\d+)" --replace "img_$1"
```

### 练习 4：text-replace

添加新工具：批量替换文件内容
```bash
$ text-replace src/ --find "TODO" --replace "DONE" --ext rs
```

---

## 扩展阅读

- [walkdir crate](https://docs.rs/walkdir) - 目录遍历
- [regex crate](https://docs.rs/regex) - 正则表达式
- [indicatif crate](https://docs.rs/indicatif) - 进度条
- [rayon crate](https://docs.rs/rayon) - 并行处理
- [ignore crate](https://docs.rs/ignore) - gitignore 风格过滤

---

## 下一章预告

文本工具集完成了，下一章进入并发编程，学习如何让程序同时做多件事，充分利用多核 CPU。
