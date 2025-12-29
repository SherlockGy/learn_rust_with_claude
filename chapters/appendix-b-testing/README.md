# 附录 B：Rust 测试完全指南

## 本附录内容

- 单元测试（Unit Tests）
- 集成测试（Integration Tests）
- 文档测试（Doc Tests）
- 测试组织与最佳实践

---

## 单元测试

### 基础结构

```rust
// src/lib.rs 或 src/任意模块.rs

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 测试模块（条件编译，只在测试时编译）
#[cfg(test)]
mod tests {
    use super::*;  // 导入父模块所有内容

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_add

# 运行包含某字符串的测试
cargo test add

# 显示 println! 输出（默认被捕获）
cargo test -- --nocapture

# 单线程运行（避免并发问题）
cargo test -- --test-threads=1

# 只运行被忽略的测试
cargo test -- --ignored

# 运行所有测试（包括被忽略的）
cargo test -- --include-ignored
```

### 断言宏

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertions() {
        // 相等断言
        assert_eq!(1 + 1, 2);
        assert_ne!(1 + 1, 3);

        // 布尔断言
        assert!(true);
        assert!(!false);

        // 自定义错误消息
        assert_eq!(
            add(2, 2), 4,
            "Expected 2 + 2 = 4, but got {}",
            add(2, 2)
        );
    }

    #[test]
    fn test_option_result() {
        let opt: Option<i32> = Some(42);
        assert!(opt.is_some());

        let res: Result<i32, &str> = Ok(42);
        assert!(res.is_ok());
    }
}
```

### 测试 panic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 期望 panic
    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("This should panic");
    }

    // 期望特定 panic 消息
    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_index_out_of_bounds() {
        let v = vec![1, 2, 3];
        v[99]; // panic: index out of bounds
    }
}
```

### 测试 Result

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 返回 Result 的测试（推荐方式）
    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        let num: i32 = "42".parse()?;
        assert_eq!(num, 42);
        Ok(())
    }

    // 测试错误情况
    #[test]
    fn test_parse_error() {
        let result: Result<i32, _> = "not a number".parse();
        assert!(result.is_err());
    }
}
```

### 忽略测试

```rust
#[cfg(test)]
mod tests {
    // 默认跳过（比如：耗时长、需要外部服务）
    #[test]
    #[ignore]
    fn expensive_test() {
        // 运行：cargo test -- --ignored
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    // 带原因的忽略
    #[test]
    #[ignore = "requires database connection"]
    fn test_database() {
        // ...
    }
}
```

### 测试私有函数

```rust
// src/lib.rs

fn internal_add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn public_add(a: i32, b: i32) -> i32 {
    internal_add(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ✓ 可以测试私有函数
    #[test]
    fn test_internal() {
        assert_eq!(internal_add(1, 2), 3);
    }
}
```

---

## 集成测试

### 目录结构

```
my_project/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/              # 集成测试目录
    ├── integration_test.rs
    ├── another_test.rs
    └── common/         # 共享测试工具
        └── mod.rs
```

### 编写集成测试

```rust
// tests/integration_test.rs

// 导入被测试的 crate
use my_project;

#[test]
fn test_public_api() {
    // 只能访问公开 API
    assert_eq!(my_project::public_add(1, 2), 3);
}

#[test]
fn test_workflow() {
    // 测试完整工作流
    let config = my_project::Config::new();
    let result = my_project::process(&config);
    assert!(result.is_ok());
}
```

### 共享测试工具

```rust
// tests/common/mod.rs

pub fn setup() -> TestContext {
    // 创建测试环境
    TestContext::new()
}

pub fn teardown(ctx: TestContext) {
    // 清理测试环境
    ctx.cleanup();
}

pub struct TestContext {
    pub temp_dir: std::path::PathBuf,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("my_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        Self { temp_dir }
    }

    pub fn cleanup(self) {
        std::fs::remove_dir_all(&self.temp_dir).ok();
    }
}
```

```rust
// tests/integration_test.rs

mod common;

#[test]
fn test_with_setup() {
    let ctx = common::setup();
    // ... 测试代码 ...
    common::teardown(ctx);
}
```

### 运行集成测试

```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定集成测试文件
cargo test --test integration_test

# 只运行单元测试（排除集成测试）
cargo test --lib
```

---

## 文档测试

### 文档测试基础

```rust
/// 将两个数相加
///
/// # Examples
///
/// ```
/// let result = my_lib::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**文档测试会被自动提取并运行！**

### 文档测试语法

```rust
/// # Examples
///
/// 基础用法：
/// ```
/// let x = 5;
/// assert_eq!(x, 5);
/// ```
///
/// 隐藏部分代码（用 # 前缀）：
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = "42".parse::<i32>()?;
/// assert_eq!(result, 42);
/// # Ok(())
/// # }
/// ```
///
/// 标记应该 panic：
/// ```should_panic
/// panic!("boom!");
/// ```
///
/// 标记不编译（用于展示错误示例）：
/// ```compile_fail
/// let x: i32 = "not a number";
/// ```
///
/// 标记不运行（只检查编译）：
/// ```no_run
/// loop {
///     // 无限循环，不要运行
/// }
/// ```
///
/// 忽略测试：
/// ```ignore
/// // 需要特殊环境
/// connect_to_production_db();
/// ```
pub fn example() {}
```

### 运行文档测试

```bash
# 运行文档测试
cargo test --doc

# 生成文档并查看
cargo doc --open
```

### 文档测试最佳实践

```rust
/// 解析配置文件
///
/// # Arguments
///
/// * `path` - 配置文件路径
///
/// # Returns
///
/// 解析成功返回 `Config`，失败返回错误
///
/// # Errors
///
/// - 文件不存在时返回 `io::Error`
/// - 格式错误时返回 `ParseError`
///
/// # Examples
///
/// ```
/// # use std::io::Write;
/// # let dir = tempfile::tempdir().unwrap();
/// # let path = dir.path().join("config.toml");
/// # std::fs::write(&path, "name = \"test\"").unwrap();
/// let config = my_lib::parse_config(&path)?;
/// assert_eq!(config.name, "test");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_config(path: &Path) -> Result<Config, Error> {
    // ...
}
```

---

## 测试组织

### 模块内测试 vs 独立文件

```rust
// 方式 1：模块内测试（推荐用于单元测试）
// src/parser.rs
pub fn parse(s: &str) -> Result<Data> {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // ...
    }
}
```

```rust
// 方式 2：独立测试文件（用于大量测试）
// src/parser.rs
pub fn parse(s: &str) -> Result<Data> {
    // ...
}

// src/parser/tests.rs 或 tests/parser_tests.rs
// 集成测试方式
```

### 测试组织最佳实践

| 测试类型 | 位置 | 适用场景 |
|---------|------|---------|
| 单元测试 | `src/*.rs` 的 `#[cfg(test)] mod tests` | 测试私有函数、细粒度逻辑 |
| 集成测试 | `tests/*.rs` | 测试公开 API、模块协作 |
| 文档测试 | 文档注释 `/// ` | API 使用示例 |

### 测试命名规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 格式：test_<function>_<scenario>_<expected>
    #[test]
    fn test_add_positive_numbers_returns_sum() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_with_zero_returns_other() {
        assert_eq!(add(0, 5), 5);
        assert_eq!(add(5, 0), 5);
    }

    #[test]
    fn test_add_negative_numbers_returns_sum() {
        assert_eq!(add(-2, -3), -5);
    }

    // 或使用模块分组
    mod add {
        use super::*;

        #[test]
        fn positive_numbers() {
            assert_eq!(super::super::add(2, 3), 5);
        }

        #[test]
        fn with_zero() {
            assert_eq!(super::super::add(0, 5), 5);
        }
    }
}
```

---

## 高级测试技巧

### 测试 fixtures（setup/teardown）

```rust
// 使用 Drop 自动清理
struct TestFixture {
    temp_file: std::path::PathBuf,
}

impl TestFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join("test_file.txt");
        std::fs::write(&path, "test data").unwrap();
        Self { temp_file: path }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        std::fs::remove_file(&self.temp_file).ok();
    }
}

#[test]
fn test_with_fixture() {
    let fixture = TestFixture::new();
    // ... 使用 fixture.temp_file ...
    // 函数结束时自动清理
}
```

### 参数化测试（使用宏）

```rust
macro_rules! test_cases {
    ($($name:ident: $input:expr => $expected:expr),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(parse($input), $expected);
            }
        )*
    };
}

test_cases! {
    parse_empty: "" => None,
    parse_number: "42" => Some(42),
    parse_negative: "-5" => Some(-5),
    parse_invalid: "abc" => None,
}
```

### 使用 test-case crate

```toml
[dev-dependencies]
test-case = "3.3"
```

```rust
use test_case::test_case;

#[test_case(0, 0 => 0; "zeros")]
#[test_case(1, 1 => 2; "ones")]
#[test_case(-1, 1 => 0; "negative and positive")]
fn test_add(a: i32, b: i32) -> i32 {
    add(a, b)
}
```

### Mock 和测试替身

```rust
// 使用 trait 实现可测试性
trait Database {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

// 生产实现
struct RealDatabase { /* ... */ }
impl Database for RealDatabase { /* ... */ }

// 测试替身
struct MockDatabase {
    data: HashMap<String, String>,
}

impl Database for MockDatabase {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
}

// 被测试的函数
fn process_data<D: Database>(db: &mut D, key: &str) {
    if let Some(value) = db.get(key) {
        db.set(&format!("{}_processed", key), value.to_uppercase());
    }
}

#[test]
fn test_process_data() {
    let mut mock = MockDatabase {
        data: HashMap::from([("test".into(), "hello".into())]),
    };

    process_data(&mut mock, "test");

    assert_eq!(mock.get("test_processed"), Some("HELLO".into()));
}
```

### 异步测试

```toml
[dev-dependencies]
tokio = { version = "1", features = ["rt", "macros"] }
```

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_fetch().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent() {
    let (a, b) = tokio::join!(task_a(), task_b());
    assert!(a.is_ok());
    assert!(b.is_ok());
}
```

---

## 测试覆盖率

### 使用 cargo-tarpaulin

```bash
# 安装
cargo install cargo-tarpaulin

# 运行覆盖率分析
cargo tarpaulin

# 生成 HTML 报告
cargo tarpaulin --out Html

# 排除某些文件
cargo tarpaulin --exclude-files "tests/*"
```

### 使用 llvm-cov

```bash
# 安装
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

# 运行
cargo llvm-cov

# 生成 HTML 报告
cargo llvm-cov --html
```

---

## 与 Java 测试对比

| 特性 | Rust | Java (JUnit) |
|-----|------|--------------|
| 测试位置 | 同文件或 tests/ | 独立 test/ 目录 |
| 测试私有方法 | ✓ 可以 | ✗ 需要反射 |
| 断言 | `assert!`、`assert_eq!` | `assertEquals` 等 |
| 期望异常 | `#[should_panic]` | `@Test(expected=)` |
| 忽略测试 | `#[ignore]` | `@Ignore` |
| 参数化测试 | 宏或 test-case crate | `@ParameterizedTest` |
| Mock | 手动或 mockall crate | Mockito 等 |
| 文档测试 | ✓ 内置 | ✗ 无 |

---

## 最佳实践清单

### 测试编写

- [ ] 每个公开函数至少一个测试
- [ ] 覆盖正常路径和边界情况
- [ ] 测试错误情况
- [ ] 使用描述性测试名称
- [ ] 保持测试独立（无共享状态）

### 测试组织

- [ ] 单元测试放在被测代码旁边
- [ ] 集成测试放在 tests/ 目录
- [ ] 共享代码放在 tests/common/
- [ ] 文档测试作为使用示例

### CI/CD

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo test --doc
```

---

## 延伸阅读

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [mockall crate](https://docs.rs/mockall/)
- [test-case crate](https://docs.rs/test-case/)
