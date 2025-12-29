# 第 16 章：文件与 I/O

## 本章目标

学完本章，你将能够：
- 使用 std::fs 进行文件读写操作
- 理解 BufReader/BufWriter 的缓冲机制
- 正确处理跨平台路径问题
- 实现 find-rs 文件查找工具

---

## 前置知识

- 第 8 章：错误处理
- 第 12 章：迭代器

---

## Rust 文件 I/O 概述

### 标准库的 I/O 模块

Rust 的文件 I/O 主要由以下模块提供：

| 模块 | 用途 |
|------|------|
| `std::fs` | 文件系统操作（读写、创建、删除） |
| `std::io` | I/O trait 和缓冲类型 |
| `std::path` | 路径处理 |

### 与 Java 对比

```java
// Java - Files API
String content = Files.readString(Path.of("file.txt"));
Files.writeString(Path.of("file.txt"), "content");

// Java - BufferedReader
BufferedReader reader = new BufferedReader(new FileReader("file.txt"));
String line;
while ((line = reader.readLine()) != null) {
    System.out.println(line);
}
```

```rust
// Rust - fs API
let content = fs::read_to_string("file.txt")?;
fs::write("file.txt", "content")?;

// Rust - BufReader
let file = File::open("file.txt")?;
let reader = BufReader::new(file);
for line in reader.lines() {
    println!("{}", line?);
}
```

**关键差异**：

| 方面 | Java | Rust |
|------|------|------|
| 错误处理 | 异常 | Result |
| 资源管理 | try-with-resources | RAII (Drop) |
| 路径类型 | Path/String | Path/PathBuf |
| 缓冲 | BufferedReader | BufReader |

---

## 项目：find-rs - 文件查找工具

### 功能概览

一个类似 Unix `find` 命令的文件查找工具：

```bash
$ find-rs . -name "*.rs"
./src/main.rs
./src/lib.rs
./tests/integration_test.rs

$ find-rs . -name "*.rs" -type f
# 只查找文件

$ find-rs ./src -name "mod.rs"
./src/utils/mod.rs
./src/handlers/mod.rs
```

### 为什么做这个项目？

1. **实用工具**：文件查找是常见需求
2. **综合练习**：路径处理、递归遍历、模式匹配
3. **跨平台思考**：处理 Windows/Unix 路径差异

---

## 核心概念

### 1. 文件读写基础

#### 一次性读写（小文件）

```rust
use std::fs;

// 读取整个文件为字符串
let content = fs::read_to_string("config.txt")?;

// 读取为字节数组
let bytes = fs::read("image.png")?;

// 写入字符串
fs::write("output.txt", "Hello, Rust!")?;

// 写入字节
fs::write("data.bin", &[0x48, 0x65, 0x6c, 0x6c, 0x6f])?;
```

**命名解释**：
- `read_to_string`：读取（read）内容转换到（to）字符串（string）
- `fs::write`：文件系统（fs）写入（write）

#### 使用 File 类型

```rust
use std::fs::File;
use std::io::{Read, Write};

// 打开文件读取
let mut file = File::open("input.txt")?;
let mut content = String::new();
file.read_to_string(&mut content)?;

// 创建文件写入
let mut file = File::create("output.txt")?;
file.write_all(b"Hello, Rust!")?;

// OpenOptions 精细控制
use std::fs::OpenOptions;

let file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)      // 追加模式
    .create(true)      // 不存在则创建
    .open("log.txt")?;
```

### 2. 缓冲读写（大文件）

直接使用 `read`/`write` 每次都是系统调用，效率低。缓冲可以显著提升性能。

#### BufReader

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

let file = File::open("large_file.txt")?;
let reader = BufReader::new(file);

// 按行读取
for line in reader.lines() {
    let line = line?;  // line 是 String
    println!("{}", line);
}
```

**为什么需要 BufReader？**

```
无缓冲：每次 read_line 都触发系统调用
┌──────┐    ┌──────┐    ┌──────┐
│ read │ -> │ read │ -> │ read │ -> ...  （频繁系统调用）
└──────┘    └──────┘    └──────┘

有缓冲：批量读取，减少系统调用
┌──────────────────────────────┐
│       BufReader 缓冲区        │
│  [数据] [数据] [数据] [数据]   │  （一次读取多行）
└──────────────────────────────┘
```

#### BufWriter

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

let file = File::create("output.txt")?;
let mut writer = BufWriter::new(file);

for i in 0..1000 {
    writeln!(writer, "Line {}", i)?;
}

// 重要：确保缓冲区内容写入文件
writer.flush()?;
// 或者让 writer 离开作用域时自动 flush
```

#### 常用 BufRead 方法

```rust
use std::io::BufRead;

let reader = BufReader::new(file);

// lines()：按行迭代，自动去掉换行符
for line in reader.lines() {
    let line = line?;
}

// read_line()：读取一行到 String
let mut line = String::new();
reader.read_line(&mut line)?;

// split()：按字节分割
for chunk in reader.split(b'\t') {
    let chunk = chunk?;
}

// read_until()：读取直到指定字节
let mut buf = Vec::new();
reader.read_until(b'\n', &mut buf)?;
```

### 3. 路径处理

Rust 使用 `Path` 和 `PathBuf` 处理路径，确保跨平台兼容。

#### Path vs PathBuf

| 类型 | 类比 | 说明 |
|------|------|------|
| `Path` | `str` | 不可变路径引用 |
| `PathBuf` | `String` | 可变路径，拥有所有权 |

```rust
use std::path::{Path, PathBuf};

// Path：不可变引用
let path = Path::new("/home/user/file.txt");

// PathBuf：拥有所有权，可修改
let mut path_buf = PathBuf::from("/home/user");
path_buf.push("documents");
path_buf.push("file.txt");
// /home/user/documents/file.txt
```

#### 路径操作

```rust
let path = Path::new("/home/user/documents/file.txt");

// 获取组成部分
path.parent();           // Some("/home/user/documents")
path.file_name();        // Some("file.txt")
path.file_stem();        // Some("file")
path.extension();        // Some("txt")

// 路径判断
path.exists();           // 是否存在
path.is_file();          // 是否是文件
path.is_dir();           // 是否是目录
path.is_absolute();      // 是否是绝对路径

// 路径拼接
let new_path = path.join("subdir");

// 修改扩展名
let backup = path.with_extension("bak");
// /home/user/documents/file.bak
```

#### 跨平台路径

```rust
use std::path::MAIN_SEPARATOR;

// Windows: '\'
// Unix: '/'
println!("路径分隔符: {}", MAIN_SEPARATOR);

// 推荐：使用 Path API，自动处理平台差异
let path = Path::new("dir").join("subdir").join("file.txt");
// Windows: dir\subdir\file.txt
// Unix: dir/subdir/file.txt

// 不推荐：字符串拼接
let bad_path = format!("dir/subdir/file.txt");  // Windows 上可能有问题
```

### 4. 目录操作

```rust
use std::fs;

// 创建目录
fs::create_dir("new_dir")?;              // 单层
fs::create_dir_all("a/b/c")?;            // 递归创建

// 删除
fs::remove_file("file.txt")?;            // 删除文件
fs::remove_dir("empty_dir")?;            // 删除空目录
fs::remove_dir_all("dir")?;              // 递归删除

// 重命名/移动
fs::rename("old.txt", "new.txt")?;

// 复制
fs::copy("src.txt", "dst.txt")?;

// 读取目录内容
for entry in fs::read_dir(".")? {
    let entry = entry?;
    println!("{:?}", entry.path());
}
```

### 5. 递归目录遍历

```rust
use std::fs;
use std::path::Path;

fn visit_dirs(dir: &Path) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path)?;
            } else {
                println!("文件: {}", path.display());
            }
        }
    }
    Ok(())
}
```

### 6. 文件元数据

```rust
use std::fs;

let metadata = fs::metadata("file.txt")?;

// 基本信息
metadata.len();           // 文件大小（字节）
metadata.is_file();       // 是否是文件
metadata.is_dir();        // 是否是目录

// 时间信息
metadata.modified()?;     // 修改时间
metadata.accessed()?;     // 访问时间
metadata.created()?;      // 创建时间（不是所有平台都支持）

// 权限（Unix）
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let permissions = metadata.permissions();
    let mode = permissions.mode();
}
```

---

## find-rs 完整实现

```rust
use std::env;
use std::fs;
use std::path::Path;

struct FindOptions {
    name_pattern: Option<String>,
    file_type: Option<FileType>,
}

#[derive(PartialEq)]
enum FileType {
    File,
    Directory,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("用法: find-rs <路径> [-name <模式>] [-type f|d]");
        return;
    }

    let start_path = Path::new(&args[1]);
    let options = parse_options(&args[2..]);

    if !start_path.exists() {
        eprintln!("错误: 路径不存在: {}", start_path.display());
        return;
    }

    find_files(start_path, &options);
}

fn parse_options(args: &[String]) -> FindOptions {
    let mut options = FindOptions {
        name_pattern: None,
        file_type: None,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-name" if i + 1 < args.len() => {
                options.name_pattern = Some(args[i + 1].clone());
                i += 2;
            }
            "-type" if i + 1 < args.len() => {
                options.file_type = match args[i + 1].as_str() {
                    "f" => Some(FileType::File),
                    "d" => Some(FileType::Directory),
                    _ => None,
                };
                i += 2;
            }
            _ => i += 1,
        }
    }

    options
}

fn find_files(dir: &Path, options: &FindOptions) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("无法读取目录 {}: {}", dir.display(), e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_dir = path.is_dir();

        // 检查类型过滤
        let type_matches = match &options.file_type {
            Some(FileType::File) => !is_dir,
            Some(FileType::Directory) => is_dir,
            None => true,
        };

        // 检查名称模式
        let name_matches = match &options.name_pattern {
            Some(pattern) => {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| matches_glob(n, pattern))
                    .unwrap_or(false)
            }
            None => true,
        };

        if type_matches && name_matches {
            println!("{}", path.display());
        }

        // 递归处理子目录
        if is_dir {
            find_files(&path, options);
        }
    }
}

/// 简单的通配符匹配
/// 支持 * 匹配任意字符
fn matches_glob(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // 处理 *.ext 模式
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return name.ends_with(&format!(".{}", ext));
    }

    // 处理 prefix* 模式
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }

    // 精确匹配
    name == pattern
}
```

---

## 错误处理模式

### 处理多个文件操作

```rust
use std::fs;
use std::path::Path;

fn process_files(paths: &[&Path]) -> Result<(), Vec<(String, std::io::Error)>> {
    let mut errors = Vec::new();

    for path in paths {
        if let Err(e) = process_single_file(path) {
            errors.push((path.display().to_string(), e));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn process_single_file(path: &Path) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    // 处理内容...
    Ok(())
}
```

### 原子写入

写入文件时，先写入临时文件再重命名，避免写入过程中断导致数据损坏：

```rust
use std::fs;
use std::path::Path;

fn safe_write(path: &Path, content: &str) -> std::io::Result<()> {
    // 1. 写入临时文件
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)?;

    // 2. 原子重命名（大多数文件系统上是原子操作）
    fs::rename(&tmp_path, path)?;

    Ok(())
}
```

---

## 最佳实践

### 选择合适的读写方式

| 场景 | 推荐方法 | 原因 |
|------|---------|------|
| 小文件（< 几 MB） | `fs::read_to_string` | 简单直接 |
| 大文件 | `BufReader` 逐行 | 避免内存爆炸 |
| 二进制文件 | `fs::read` / `File::read` | 返回 `Vec<u8>` |
| 追加写入 | `OpenOptions::append` | 不覆盖原内容 |
| 批量小写入 | `BufWriter` | 减少系统调用 |

### 路径处理

| 场景 | 推荐 | 避免 |
|------|------|------|
| 路径拼接 | `path.join("subdir")` | 字符串拼接 `+` |
| 临时路径 | `PathBuf` | 频繁 `Path::new` |
| 跨平台 | `Path` API | 硬编码分隔符 |

### 常见陷阱

| 陷阱 | 问题 | 解决方案 |
|------|------|---------|
| 忘记 flush | BufWriter 内容未写入 | 显式调用 `flush()` 或让其 drop |
| 路径编码 | 非 UTF-8 路径 | 使用 `OsStr` / `OsString` |
| 竞态条件 | exists() 检查后文件被删 | 直接操作，处理错误 |
| 文件句柄泄露 | 文件未关闭 | 依赖 RAII 或显式 drop |

```rust
// 竞态条件示例
// 不好：检查和创建之间可能有其他进程操作
if !path.exists() {
    fs::create_dir(&path)?;  // 可能失败：目录已被其他进程创建
}

// 好：直接尝试创建，处理错误
match fs::create_dir(&path) {
    Ok(()) => println!("目录已创建"),
    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
        println!("目录已存在");
    }
    Err(e) => return Err(e),
}
```

---

## 要点回顾

1. **简单读写**用 `fs::read_to_string` / `fs::write`
2. **大文件**用 `BufReader` / `BufWriter`
3. **路径处理**用 `Path` / `PathBuf`，不要字符串拼接
4. **跨平台**使用 Path API，避免硬编码分隔符
5. **原子写入**先写临时文件再重命名
6. **资源管理**依赖 RAII，文件自动关闭

---

## 练习

### 练习 1：目录大小统计

实现一个统计目录总大小的工具，递归计算所有文件大小：

```bash
$ dir-size ./src
总大小: 1.2 MB (1,234,567 字节)
文件数: 42
```

### 练习 2：文件类型过滤

为 find-rs 添加按扩展名过滤的功能：

```bash
$ find-rs . -ext rs,toml
```

### 练习 3：Tree 工具

实现类似 `tree` 的目录树展示工具：

```bash
$ tree-rs ./src
src/
├── main.rs
├── lib.rs
└── utils/
    ├── mod.rs
    └── helper.rs
```

### 练习 4：文件备份

实现一个文件备份工具，支持：
- 增量备份（只复制修改过的文件）
- 保留目录结构

---

## 扩展阅读

- [std::fs 文档](https://doc.rust-lang.org/std/fs/)
- [std::path 文档](https://doc.rust-lang.org/std/path/)
- [std::io 文档](https://doc.rust-lang.org/std/io/)
- [walkdir crate](https://docs.rs/walkdir) - 更强大的目录遍历
- [glob crate](https://docs.rs/glob) - 通配符匹配
- [tempfile crate](https://docs.rs/tempfile) - 临时文件处理

---

## 下一章预告

文件操作学会了，下一章使用 Workspace 组织多个文本处理小工具，构建一个实用的工具集。
