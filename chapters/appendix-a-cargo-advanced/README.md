# 附录 A：Cargo 进阶

## 本附录内容

- Feature Flags（条件编译）
- build.rs（构建脚本）
- 发布到 crates.io
- Cargo 配置与优化

---

## Feature Flags：条件编译

### 什么是 Feature？

Feature 是 Cargo 提供的条件编译机制，允许：
- 可选依赖（按需引入）
- 条件代码（编译时选择）
- 减小最终二进制体积

**Java 对比**：类似 Maven 的 profile，但更细粒度。

### 定义 Feature

```toml
# Cargo.toml
[package]
name = "my-lib"
version = "1.0.0"

[features]
# 默认启用的 feature
default = ["json"]

# 基础 feature
json = ["dep:serde_json"]
yaml = ["dep:serde_yaml"]
full = ["json", "yaml"]  # 组合 feature

# 可选依赖自动成为 feature
[dependencies]
serde = "1.0"
serde_json = { version = "1.0", optional = true }
serde_yaml = { version = "0.9", optional = true }
```

### Feature 命名规范

```toml
[features]
# ✓ 好的命名
json = []           # 格式支持
async = []          # 功能开关
derive = []         # derive 宏支持

# ✗ 避免的命名
no-std = []         # 改用 std（默认） + 可选禁用
enable-json = []    # 冗余前缀
```

### 在代码中使用 Feature

```rust
// 条件编译
#[cfg(feature = "json")]
pub mod json {
    use serde_json;

    pub fn parse(s: &str) -> Result<Value, Error> {
        serde_json::from_str(s)
    }
}

// 条件导入
#[cfg(feature = "json")]
use serde_json::Value;

// 条件实现
impl MyStruct {
    #[cfg(feature = "async")]
    pub async fn fetch(&self) -> Result<Data> {
        // 异步实现
    }

    #[cfg(not(feature = "async"))]
    pub fn fetch(&self) -> Result<Data> {
        // 同步实现
    }
}

// 条件 derive
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub name: String,
}
```

### 使用带 Feature 的依赖

```toml
# 依赖方 Cargo.toml

# 默认 feature
[dependencies]
my-lib = "1.0"

# 禁用默认 feature
[dependencies]
my-lib = { version = "1.0", default-features = false }

# 启用特定 feature
[dependencies]
my-lib = { version = "1.0", features = ["json", "yaml"] }

# 禁用默认 + 启用特定
[dependencies]
my-lib = { version = "1.0", default-features = false, features = ["json"] }
```

### Feature 最佳实践

| 原则 | 说明 |
|-----|------|
| 加法原则 | Feature 只应增加功能，不应移除 |
| 默认最小 | default 只含核心功能 |
| 文档说明 | 每个 feature 在 README 中说明 |
| 测试覆盖 | CI 测试所有 feature 组合 |

---

## build.rs：构建脚本

### 什么是 build.rs？

`build.rs` 是 Cargo 在编译前运行的 Rust 脚本，用于：
- 代码生成
- 编译 C/C++ 代码（FFI）
- 设置环境变量
- 链接系统库

### 基础示例

```rust
// build.rs（放在项目根目录，与 Cargo.toml 同级）
fn main() {
    // 告诉 Cargo：如果这个文件变化，重新运行 build.rs
    println!("cargo:rerun-if-changed=build.rs");

    // 设置环境变量，代码中用 env!() 访问
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now());
}
```

```rust
// src/main.rs
fn main() {
    // 编译时嵌入
    println!("Built at: {}", env!("BUILD_TIME"));
}
```

### build.rs 指令

```rust
// build.rs
fn main() {
    // 重新运行条件
    println!("cargo:rerun-if-changed=src/proto/");
    println!("cargo:rerun-if-env-changed=MY_VAR");

    // 设置编译环境变量
    println!("cargo:rustc-env=VERSION=1.0.0");

    // 添加链接库
    println!("cargo:rustc-link-lib=sqlite3");
    println!("cargo:rustc-link-search=/usr/local/lib");

    // 启用 cfg 标志
    println!("cargo:rustc-cfg=has_feature_x");

    // 警告信息（编译时显示）
    println!("cargo:warning=Building with experimental features");
}
```

### 实用场景：嵌入 Git 版本

```rust
// build.rs
use std::process::Command;

fn main() {
    // 获取 git commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to get git hash");

    let git_hash = String::from_utf8_lossy(&output.stdout);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());

    // 获取 git 分支
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get git branch");

    let git_branch = String::from_utf8_lossy(&output.stdout);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch.trim());

    // 文件变化时重新运行
    println!("cargo:rerun-if-changed=.git/HEAD");
}
```

```rust
// src/main.rs
fn main() {
    println!("Version: {} ({})",
        env!("GIT_HASH"),
        env!("GIT_BRANCH")
    );
}
```

### build.rs 依赖

```toml
# Cargo.toml
[build-dependencies]
cc = "1.0"           # 编译 C 代码
prost-build = "0.12" # 生成 protobuf 代码
chrono = "0.4"       # 时间处理
```

---

## 发布到 crates.io

### 准备工作

1. **注册账号**：访问 https://crates.io 用 GitHub 登录

2. **获取 API Token**：
   - 登录后点击头像 → Account Settings → API Tokens
   - 创建新 token

3. **配置 Cargo**：
```bash
cargo login <your-api-token>
```

### Cargo.toml 必填字段

```toml
[package]
name = "my-awesome-crate"
version = "0.1.0"
edition = "2021"

# 发布必填
description = "A short description of what this crate does"
license = "MIT OR Apache-2.0"
repository = "https://github.com/username/repo"

# 推荐填写
readme = "README.md"
keywords = ["cli", "tool", "utility"]  # 最多 5 个
categories = ["command-line-utilities"] # 见 crates.io/categories
authors = ["Your Name <email@example.com>"]
documentation = "https://docs.rs/my-awesome-crate"
homepage = "https://github.com/username/repo"

# 排除不需要发布的文件
exclude = [
    "tests/",
    "benches/",
    ".github/",
    "*.md",
    "!README.md",
]
```

### 版本号规范（SemVer）

```
主版本.次版本.修订版本
  1   .  2  .   3
```

| 变更类型 | 版本号变化 | 示例 |
|---------|-----------|------|
| 破坏性变更（API 改变）| 主版本 +1 | 1.2.3 → 2.0.0 |
| 新功能（向后兼容）| 次版本 +1 | 1.2.3 → 1.3.0 |
| Bug 修复 | 修订版本 +1 | 1.2.3 → 1.2.4 |

**0.x.y 特殊规则**：
- `0.1.0` → `0.2.0`：可能有破坏性变更
- 正式发布前用 0.x，API 稳定后发布 1.0

### 发布流程

```bash
# 1. 检查 Cargo.toml 完整性
cargo publish --dry-run

# 2. 运行测试
cargo test

# 3. 构建文档
cargo doc --open

# 4. 发布！
cargo publish

# 5. 打 Git tag（推荐）
git tag v0.1.0
git push origin v0.1.0
```

### 发布检查清单

- [ ] README.md 包含使用说明和示例
- [ ] CHANGELOG.md 记录版本变更
- [ ] LICENSE 文件存在
- [ ] 所有公开 API 有文档注释
- [ ] `cargo clippy` 无警告
- [ ] `cargo test` 全部通过
- [ ] `cargo doc` 无警告

### 撤回版本（Yank）

```bash
# 撤回某版本（不删除，但不会被新项目依赖）
cargo yank --version 0.1.0

# 取消撤回
cargo yank --version 0.1.0 --undo
```

**注意**：crates.io 不支持删除已发布版本，只能 yank。

---

## Cargo 配置与优化

### .cargo/config.toml

```toml
# .cargo/config.toml（项目级或 ~/.cargo/config.toml 全局）

# 默认使用的 target（交叉编译）
[build]
target = "x86_64-unknown-linux-gnu"

# 别名
[alias]
b = "build"
t = "test"
r = "run"
rr = "run --release"
c = "clippy -- -D warnings"

# 环境变量
[env]
RUST_BACKTRACE = "1"

# 网络配置（国内镜像）
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

# 链接器配置（加速链接）
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Release 构建优化

```toml
# Cargo.toml

[profile.release]
# 链接时优化（体积更小，编译更慢）
lto = true

# 优化级别：0-3，s（体积），z（最小体积）
opt-level = 3

# 代码生成单元（1 = 最优，但编译慢）
codegen-units = 1

# 移除调试符号
strip = true

# panic 处理（abort 更小）
panic = "abort"
```

### 自定义 Profile

```toml
# Cargo.toml

# 用于性能测试的 profile
[profile.bench]
inherits = "release"
debug = true  # 保留调试符号，方便 profiling

# 用于 CI 的快速检查
[profile.ci]
inherits = "dev"
opt-level = 0
debug = false

# 生产部署优化
[profile.production]
inherits = "release"
lto = "fat"
codegen-units = 1
strip = true
```

```bash
# 使用自定义 profile
cargo build --profile production
```

### Cargo 实用命令

```bash
# 查看依赖树
cargo tree

# 查看过时的依赖
cargo outdated  # 需安装 cargo-outdated

# 安全审计
cargo audit  # 需安装 cargo-audit

# 检查未使用的依赖
cargo udeps  # 需安装 cargo-udeps

# 生成许可证报告
cargo license  # 需安装 cargo-license

# 二进制体积分析
cargo bloat  # 需安装 cargo-bloat
```

### 常用 Cargo 插件

| 插件 | 用途 | 安装命令 |
|-----|------|---------|
| cargo-watch | 文件变化自动运行 | `cargo install cargo-watch` |
| cargo-edit | 命令行管理依赖 | `cargo install cargo-edit` |
| cargo-expand | 展开宏代码 | `cargo install cargo-expand` |
| cargo-outdated | 检查过时依赖 | `cargo install cargo-outdated` |
| cargo-audit | 安全漏洞检查 | `cargo install cargo-audit` |
| cargo-flamegraph | 性能火焰图 | `cargo install flamegraph` |

```bash
# cargo-watch 示例
cargo watch -x "test" -x "run"

# cargo-edit 示例
cargo add serde --features derive
cargo rm unused-crate

# cargo-expand 示例
cargo expand --lib
```

---

## 快速参考

### Feature 语法速查

```toml
# 定义
[features]
default = ["a"]
a = []
b = ["dep:optional_dep"]
c = ["a", "b"]

# 使用
#[cfg(feature = "a")]
#[cfg(all(feature = "a", feature = "b"))]
#[cfg(any(feature = "a", feature = "b"))]
#[cfg(not(feature = "a"))]
```

### build.rs 指令速查

```rust
println!("cargo:rerun-if-changed=PATH");
println!("cargo:rerun-if-env-changed=VAR");
println!("cargo:rustc-env=KEY=VALUE");
println!("cargo:rustc-cfg=FLAG");
println!("cargo:rustc-link-lib=NAME");
println!("cargo:rustc-link-search=PATH");
println!("cargo:warning=MESSAGE");
```

### 发布命令速查

```bash
cargo publish --dry-run  # 检查
cargo publish            # 发布
cargo yank --version X   # 撤回
cargo owner --add USER   # 添加维护者
```

---

## 延伸阅读

- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [crates.io Policies](https://crates.io/policies)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
