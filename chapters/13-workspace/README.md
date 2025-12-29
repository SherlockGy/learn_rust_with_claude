# 第 13 章：Cargo Workspace 与项目组织

## 本章目标

学完本章，你将能够：
- 创建和配置 workspace
- 管理多个相关项目
- 共享依赖版本
- 组织大型 Rust 项目

---

## 前置知识

- 第 1-12 章的所有项目

---

## 为什么需要 Workspace

随着项目增长，你可能会遇到这些问题：

1. **多个相关项目**：task-cli、freq、mini-vec 等都是独立项目，但经常一起开发
2. **重复依赖**：每个项目都依赖 serde 1.0，版本分散在各处
3. **共享代码**：想提取公共模块，但不想发布到 crates.io
4. **构建效率**：每个项目单独编译，依赖重复下载

**Workspace** 解决这些问题：
- 统一管理多个 crate
- 共享 `target` 目录和 `Cargo.lock`
- 统一依赖版本
- 内部 crate 互相引用

---

## 项目：整合已有项目为 workspace

### 最终结构

```
learn-rust-projects/
├── Cargo.toml          # workspace 根配置
├── Cargo.lock          # 共享的锁文件
├── target/             # 共享的构建目录
├── echo-rs/
│   ├── Cargo.toml
│   └── src/main.rs
├── word-count/
│   ├── Cargo.toml
│   └── src/main.rs
├── task-cli/
│   ├── Cargo.toml
│   └── src/main.rs
├── freq/
│   ├── Cargo.toml
│   └── src/main.rs
└── common/             # 共享库
    ├── Cargo.toml
    └── src/lib.rs
```

---

## 实操步骤

### 步骤 1：创建 workspace 根目录

```bash
mkdir learn-rust-projects
cd learn-rust-projects
```

### 步骤 2：创建根 Cargo.toml

```bash
# 创建 workspace 配置文件
```

```toml
# Cargo.toml（根目录）
[workspace]
members = [
    "echo-rs",
    "word-count",
    "task-cli",
    "freq",
    "common",
]

# 统一管理依赖版本
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4", features = ["derive"] }
```

**注意**：根 Cargo.toml 只有 `[workspace]` 部分，没有 `[package]`。

### 步骤 3：移动现有项目

```bash
# 假设原项目在其他位置
mv ~/projects/echo-rs ./
mv ~/projects/word-count ./
mv ~/projects/task-cli ./
mv ~/projects/freq ./
```

### 步骤 4：修改子项目使用 workspace 依赖

```toml
# task-cli/Cargo.toml
[package]
name = "task-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
# 使用 workspace 统一版本
serde = { workspace = true }
serde_json = { workspace = true }
clap = { workspace = true }
```

**关键语法**：`{ workspace = true }` 表示从根 workspace 继承版本。

### 步骤 5：创建共享库

```bash
cargo new common --lib
```

```toml
# common/Cargo.toml
[package]
name = "common"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
```

```rust
// common/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: String,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: String::from("."),
            verbose: false,
        }
    }
}

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
```

### 步骤 6：在其他项目中使用共享库

```toml
# freq/Cargo.toml
[package]
name = "freq"
version = "0.1.0"
edition = "2021"

[dependencies]
common = { path = "../common" }
```

```rust
// freq/src/main.rs
use common::{format_size, Config};

fn main() {
    let config = Config::default();
    println!("数据目录: {}", config.data_dir);
    println!("文件大小: {}", format_size(1_500_000));
}
```

### 步骤 7：验证 workspace

```bash
# 在根目录执行
cargo build
# 所有成员一起构建

cargo check
# 快速检查所有成员

cargo test
# 运行所有成员的测试
```

---

## 核心概念

### 1. Workspace 配置选项

```toml
[workspace]
# 成员列表
members = [
    "crate-a",
    "crate-b",
    "tools/*",      # 支持通配符
]

# 排除某些目录
exclude = [
    "experimental",
]

# 依赖解析器版本（推荐 2）
resolver = "2"
```

### 2. workspace.dependencies 详解

```toml
[workspace.dependencies]
# 简单版本
regex = "1.10"

# 带 features
serde = { version = "1.0", features = ["derive"] }

# 指定可选
tokio = { version = "1", features = ["full"], optional = true }

# Git 依赖
my-crate = { git = "https://github.com/user/repo" }
```

子项目继承时可以添加额外 features：

```toml
# 子项目 Cargo.toml
[dependencies]
serde = { workspace = true, features = ["rc"] }  # 追加 rc feature
```

### 3. Workspace 命令

```bash
# 构建
cargo build                  # 构建所有成员
cargo build -p task-cli      # 只构建 task-cli
cargo build --workspace      # 显式指定构建所有

# 运行
cargo run -p echo-rs         # 运行 echo-rs
cargo run -p echo-rs -- arg  # 传递参数

# 测试
cargo test                   # 测试所有
cargo test -p common         # 只测试 common
cargo test --workspace       # 显式测试所有

# 检查
cargo check -p freq          # 快速检查 freq
cargo clippy --workspace     # lint 所有成员

# 文档
cargo doc --workspace        # 生成所有文档
cargo doc -p common --open   # 生成并打开 common 文档
```

### 4. 成员间依赖

```toml
# app/Cargo.toml
[dependencies]
# 路径依赖（开发时）
common = { path = "../common" }

# 发布时可以切换为版本依赖
# common = "0.1"
```

**依赖图示例**：
```
task-cli ──────┐
               │
freq ──────────┼──▶ common
               │
word-count ────┘
```

---

## 实用技巧

### 1. 新建成员

```bash
# 在 workspace 根目录
cargo new my-tool           # 创建二进制项目
cargo new my-lib --lib      # 创建库项目

# 然后添加到 members 列表
```

### 2. 统一 Rust 版本

```toml
# 根 Cargo.toml
[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT"
authors = ["Your Name <you@example.com>"]
```

```toml
# 子项目继承
[package]
name = "my-tool"
version = "0.1.0"
edition.workspace = true
license.workspace = true
authors.workspace = true
```

### 3. 统一 lints 配置

```toml
# 根 Cargo.toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
```

```toml
# 子项目继承
[lints]
workspace = true
```

### 4. 查看依赖树

```bash
cargo tree                   # 查看依赖树
cargo tree -p task-cli       # 指定成员
cargo tree --duplicates      # 查看重复依赖
```

---

## 最佳实践

### 何时使用 Workspace

| 场景 | 推荐 | 原因 |
|------|------|------|
| 多个相关 CLI 工具 | 用 workspace | 共享依赖和代码 |
| 库 + 示例应用 | 用 workspace | 紧密关联 |
| 微服务架构 | 用 workspace | 统一管理 |
| 单个独立项目 | 不需要 | 过度设计 |
| 完全独立的项目 | 不需要 | 无共享收益 |

### 目录组织建议

```
my-workspace/
├── Cargo.toml
├── apps/               # 应用程序
│   ├── cli/
│   └── server/
├── libs/               # 内部库
│   ├── core/
│   └── utils/
└── tools/              # 开发工具
    └── codegen/
```

对应配置：
```toml
[workspace]
members = [
    "apps/*",
    "libs/*",
    "tools/*",
]
```

### 版本管理策略

| 策略 | 适用场景 |
|------|---------|
| 统一版本 | 紧密耦合的组件 |
| 独立版本 | 可独立发布的库 |
| 锁定版本 | 生产环境稳定性 |

---

## 与 Maven 多模块项目对比

如果你熟悉 Java Maven，Cargo Workspace 的概念会很容易理解：

### 结构对比

```
Maven 多模块项目                    Cargo Workspace
─────────────────                   ───────────────
parent-project/                     my-workspace/
├── pom.xml         (父 POM)        ├── Cargo.toml    (workspace 配置)
├── module-a/                       ├── crate-a/
│   └── pom.xml                     │   └── Cargo.toml
├── module-b/                       ├── crate-b/
│   └── pom.xml                     │   └── Cargo.toml
└── common/                         └── common/
    └── pom.xml                         └── Cargo.toml
```

### 配置对比

**Maven 父 POM**：
```xml
<project>
    <groupId>com.example</groupId>
    <artifactId>parent</artifactId>
    <packaging>pom</packaging>

    <modules>
        <module>module-a</module>
        <module>module-b</module>
        <module>common</module>
    </modules>

    <dependencyManagement>
        <dependencies>
            <dependency>
                <groupId>com.fasterxml.jackson.core</groupId>
                <artifactId>jackson-databind</artifactId>
                <version>2.15.0</version>
            </dependency>
        </dependencies>
    </dependencyManagement>
</project>
```

**Cargo Workspace**：
```toml
[workspace]
members = [
    "crate-a",
    "crate-b",
    "common",
]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
```

### 子模块继承对比

**Maven 子模块**：
```xml
<project>
    <parent>
        <groupId>com.example</groupId>
        <artifactId>parent</artifactId>
        <version>1.0</version>
    </parent>

    <artifactId>module-a</artifactId>

    <dependencies>
        <!-- 版本从 parent 继承 -->
        <dependency>
            <groupId>com.fasterxml.jackson.core</groupId>
            <artifactId>jackson-databind</artifactId>
        </dependency>
    </dependencies>
</project>
```

**Cargo 子 crate**：
```toml
[package]
name = "crate-a"
version = "0.1.0"

[dependencies]
# 版本从 workspace 继承
serde = { workspace = true }
```

### 核心概念映射

| Maven | Cargo Workspace | 说明 |
|-------|-----------------|------|
| `<modules>` | `members = [...]` | 声明子模块列表 |
| `<dependencyManagement>` | `[workspace.dependencies]` | 统一依赖版本 |
| `<parent>` | `{ workspace = true }` | 继承父级配置 |
| `mvn install` | `cargo build` | 构建所有模块 |
| `mvn -pl module-a` | `cargo build -p crate-a` | 构建指定模块 |
| `<properties>` | `[workspace.package]` | 共享属性 |
| 内部依赖 `<dependency>` | `{ path = "../common" }` | 模块间引用 |

### 关键差异

| 特性 | Maven | Cargo Workspace |
|------|-------|-----------------|
| 配置格式 | XML（冗长） | TOML（简洁） |
| 构建产物 | 各模块独立 `target/` | **共享 `target/`** |
| 锁文件 | 无（或各自） | **共享 `Cargo.lock`** |
| 版本继承 | 需要显式 `<parent>` | 自动识别 workspace |
| 发布 | 可统一发布 | 各 crate 独立发布 |
| 循环依赖 | 允许（不推荐） | **编译器禁止** |

### Cargo 的优势

1. **共享构建缓存**：所有成员共用 `target/`，依赖只编译一次
2. **统一锁文件**：`Cargo.lock` 保证所有成员依赖版本一致
3. **配置更简洁**：TOML vs XML，差距明显
4. **编译时检查**：循环依赖在编译期就会报错

### 迁移提示

| Maven 习惯 | Cargo 对应做法 |
|-----------|---------------|
| `mvn clean` | `cargo clean` |
| `mvn test` | `cargo test` |
| `mvn package -DskipTests` | `cargo build --release` |
| `mvn dependency:tree` | `cargo tree` |
| 私有仓库 Nexus | 私有 registry 或 git 依赖 |

---

## 常见问题

### Q: workspace.dependencies 和直接写版本有什么区别？

```toml
# 方式 1：直接写（各项目可能版本不一致）
[dependencies]
serde = "1.0.193"

# 方式 2：workspace 继承（保证版本一致）
[dependencies]
serde = { workspace = true }
```

**方式 2 的优势**：
- 版本统一，避免依赖冲突
- 升级时只需改一处
- 编译更快（共享编译结果）

### Q: 如何处理循环依赖？

Cargo 不允许循环依赖。解决方案：
1. 提取公共代码到新 crate
2. 重新设计依赖关系
3. 使用 trait 解耦

### Q: workspace 成员可以有不同 edition 吗？

可以，每个成员可以独立设置 edition：

```toml
# 成员 A
[package]
edition = "2021"

# 成员 B（旧项目）
[package]
edition = "2018"
```

---

## 要点回顾

1. **workspace** 让多个 crate 共享构建目录
2. **workspace.dependencies** 统一依赖版本
3. **path 依赖**让成员间可以相互引用
4. **-p 参数**指定操作哪个成员
5. **cargo new** 在 workspace 中创建新成员

---

## 练习

1. **基础**：创建一个包含两个成员的 workspace
2. **进阶**：提取公共代码为内部 crate，让其他成员依赖它
3. **挑战**：配置 workspace 统一管理所有成员的 lints 和 rustfmt

---

## 扩展阅读

- [The Cargo Book - Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Cargo Reference - Workspace Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace)
- [Workspace Package Keys](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table)

---

## 下一章预告

Workspace 组织好了，下一章学习 Serde——Rust 生态最独特的序列化方案。
