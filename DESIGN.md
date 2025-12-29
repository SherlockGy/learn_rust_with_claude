# Learn Rust with Claude - 设计计划

本文档记录教程设计的决策逻辑、项目选择理由和实施细节。

---

## 1. 整体设计哲学

### 1.1 项目选择原则

每个项目必须满足：

1. **实用性**：完成后能在真实场景使用
2. **适度复杂**：足够展示本章概念，不过度复杂
3. **递进性**：复用或扩展前面章节的项目
4. **Unix 风格**：短小精悍，做好一件事

### 1.2 概念引入原则

1. **用到再讲**：不提前灌输"以后会用到"的知识
2. **最小完整**：讲清楚能用起来，细节按需补充
3. **意图先行**：先说为什么有这个设计，再说怎么用
4. **对比明确**：与 Java 的异同要明确指出

---

## 2. 项目功能概览与需求分析

每个项目在开始前，必须让学习者清楚：**要做什么、为什么做、最终效果是什么**。

### 2.1 CLI 工具线项目需求

#### echo-rs（第 1 章）
```text
功能：将命令行参数原样输出
场景：最基础的 Unix 工具，理解命令行交互

使用示例：
$ echo-rs Hello World
Hello World

$ echo-rs -n "No newline"
No newline$

需求清单：
✓ 输出所有参数，空格分隔
✓ 支持 -n 选项（不输出换行符）
```

#### word-count（第 2-3 章）
```text
功能：统计文本的行数、单词数、字符数
场景：类似 wc 命令，文本分析的基础工具

使用示例：
$ echo "Hello World" | word-count
      1       2      12

$ word-count file.txt
     42     256    1832 file.txt

需求清单：
✓ 统计行数、单词数、字符数
✓ 支持标准输入和文件参数
✓ 格式化对齐输出
```

#### uniq-rs（第 4-5 章）
```text
功能：去除或统计连续重复的行
场景：日志处理、数据清洗

使用示例：
$ cat data.txt | uniq-rs
apple
banana
apple

$ cat data.txt | uniq-rs -c
      3 apple
      1 banana
      2 apple

需求清单：
✓ 去除连续重复行
✓ -c 选项：显示重复次数
✓ 支持标准输入和文件
```

#### freq（第 12 章）
```text
功能：统计文本中每个单词出现的频率
场景：文本分析、词频统计、简易搜索引擎基础

使用示例：
$ freq article.txt
the     142
and     98
rust    67
...

$ freq --top 10 article.txt
(只显示前 10 个高频词)

需求清单：
✓ 读取文件统计词频
✓ 按频率降序排列
✓ --top N 选项限制输出数量
✓ 忽略大小写（可选）
```

#### find-rs（第 16 章）
```text
功能：在目录中查找匹配的文件
场景：文件系统操作，替代简单的 find 命令

使用示例：
$ find-rs . -name "*.rs"
./src/main.rs
./src/lib.rs

$ find-rs /home -type f -ext txt
(查找所有 txt 文件)

需求清单：
✓ 递归遍历目录
✓ 按文件名模式过滤
✓ 按扩展名过滤
✓ 按文件类型过滤（文件/目录）
```

### 2.2 Task CLI 完整需求（贯穿 6-9, 11, 14-15 章）

```text
项目名称：task-cli
定位：命令行待办事项管理器
类比：Todoist/Things/todo.txt 的命令行版本

=== 为什么做这个项目？===
- 程序员常在终端工作，需要不离开终端的任务管理
- 轻量快速，启动无延迟
- 数据本地存储，完全可控
- 可通过脚本自动化

=== 最终完整功能（第 15 章）===

$ task add "学习 Rust 所有权" --due tomorrow
✓ 任务已添加 (ID: 1)

$ task add "写周报" --priority high
✓ 任务已添加 (ID: 2)

$ task list
ID  优先级  状态    截止日期    任务
1   中      待办    明天        学习 Rust 所有权
2   高      待办    -           写周报

$ task list --status done
ID  优先级  状态    完成日期    任务
3   中      完成    2025-01-28  安装 Rust

$ task list --filter "priority=high"
(使用闭包过滤)

$ task done 1
✓ 任务 #1 已完成

$ task edit 2 --priority low
✓ 任务 #2 已更新

$ task remove 2
确定删除任务 #2 吗？(y/N) y
✓ 任务已删除

$ task stats
总计: 10 个任务
待办: 5 | 进行中: 2 | 已完成: 3

=== 功能拆分到各章节 ===

第 6 章（结构体）：
- Task { id: u32, title: String, done: bool }
- 内存中的 Vec<Task>
- 简单的 add/list/done

第 7 章（枚举）：
- Status 枚举（Pending/InProgress/Done）
- Priority 枚举（Low/Medium/High）
- Option<String> 可选截止日期

第 8 章（错误处理）：
- 保存到纯文本文件
- 处理文件读写错误
- 任务 ID 不存在的错误

第 9 章（Trait）：
- Display trait 美化输出
- Debug trait 用于调试
- 自定义格式化

第 11 章（闭包）：
- 添加 --filter 过滤功能
- 使用闭包实现灵活过滤

第 14 章（Serde）：
- JSON 格式存储任务
- TOML 配置文件（数据目录等）

第 15 章（Clap）：
- 完整子命令：add/list/done/edit/remove/stats
- 参数选项：--due, --priority, --status, --filter
- 自动帮助信息
```

### 2.3 mini-vec 需求（第 10 章）

```text
项目名称：mini-vec
定位：简化版动态数组，学习泛型的教学项目
类比：std::vec::Vec 的简化版

=== 为什么做这个项目？===
- 泛型最好的学习方式是实现一个泛型容器
- Vec 是最常用的集合，理解其原理很重要
- 通过实现加深对所有权和借用的理解

=== 功能需求 ===

使用示例：
let mut vec: MiniVec<i32> = MiniVec::new();
vec.push(1);
vec.push(2);
vec.push(3);

assert_eq!(vec.get(0), Some(&1));
assert_eq!(vec.pop(), Some(3));
assert_eq!(vec.len(), 2);

for item in &vec {
    println!("{}", item);
}

需求清单：
✓ 泛型结构体 MiniVec<T>
✓ new() 创建空向量
✓ push(item: T) 添加元素
✓ pop() -> Option<T> 移除并返回最后一个元素
✓ get(index: usize) -> Option<&T> 获取元素引用
✓ len() -> usize 返回长度
✓ is_empty() -> bool 判断是否为空

扩展（可选）：
○ 实现 Index trait
○ 实现迭代器
○ 容量管理（grow）
```

### 2.4 text-toolkit 需求（第 17 章）

```text
项目名称：text-toolkit
定位：批量文本处理工具集
场景：日常文件管理、批量操作、数据清洗

项目组织：使用 workspace 管理多个子工具
text-toolkit/
├── Cargo.toml          # workspace 根配置
├── common/             # 共享库（文件操作、输出格式化）
├── batch-rename/       # 子工具 1
├── line-stats/         # 子工具 2
└── ...

=== 子工具 1：batch-rename ===
批量重命名文件

$ batch-rename "*.jpg" --pattern "photo_" --replace "img_"
预览模式：
  photo_001.jpg -> img_001.jpg
  photo_002.jpg -> img_002.jpg
确认执行？(y/N)

需求：
✓ 支持 glob 模式匹配
✓ 正则表达式替换
✓ 预览模式（dry-run）
✓ 序号重排

=== 子工具 2：line-stats ===
统计多文件的行信息

$ line-stats src/**/*.rs
文件                    行数    空行    代码行
src/main.rs            156     23      133
src/lib.rs             89      12      77
src/utils/mod.rs       45      8       37
---
总计                   290     43      247

需求：
✓ 递归统计目录
✓ 区分代码行、空行、注释行
✓ 汇总统计

=== 子工具 3：text-replace（扩展练习）===
批量文本替换

=== 子工具 4：csv-query（扩展练习）===
简易 CSV 查询
```

### 2.5 并发项目需求（第 18-19 章）

#### parallel-hash（第 18 章）
```text
功能：并行计算多个文件的哈希值
场景：大量文件的完整性校验、重复文件检测

使用示例：
$ parallel-hash *.txt
file1.txt  sha256:a1b2c3...
file2.txt  sha256:d4e5f6...
file3.txt  sha256:g7h8i9...
完成：3 个文件，用时 0.5 秒

$ parallel-hash --algorithm md5 --threads 4 large_dir/
(使用 4 线程计算目录下所有文件的 MD5)

需求清单：
✓ 多线程并行计算
✓ 支持多种哈希算法（SHA256、MD5 等）
✓ 进度显示
✓ 结果排序输出
```

#### log-watcher（第 19 章）
```text
功能：实时监控多个日志文件，聚合输出匹配的行
场景：多服务日志聚合、实时错误监控

使用示例：
$ log-watcher /var/log/app1.log /var/log/app2.log --pattern "ERROR"
[app1.log 10:23:45] ERROR: Connection timeout
[app2.log 10:23:47] ERROR: Database query failed
[app1.log 10:24:01] ERROR: Retry limit exceeded
^C
监控结束，共匹配 3 条

$ log-watcher logs/*.log --pattern "user_id=\d+" --highlight
(监控所有日志，高亮显示包含 user_id 的行)

需求清单：
✓ 同时监控多个文件
✓ 正则表达式过滤
✓ 实时输出（tail -f 效果）
✓ 文件来源标识
✓ 优雅退出（Ctrl+C）
```

---

### 2.6 网络项目需求（第 20-23 章）

#### kv-server（第 20-22 章）
```text
项目名称：kv-server
定位：简单的 TCP 键值存储服务
类比：极简版 Redis

协议设计（文本协议）：
SET key value\n  -> OK\n 或 ERROR message\n
GET key\n        -> VALUE value\n 或 NOT_FOUND\n
DEL key\n        -> OK\n 或 NOT_FOUND\n
KEYS\n           -> KEY key1\nKEY key2\n...\nEND\n

使用示例：
# 启动服务器
$ kv-server --port 7878

# 客户端连接（可用 telnet 或 nc）
$ nc localhost 7878
SET name Alice
OK
GET name
VALUE Alice
DEL name
OK

演进路径：
- 第 20 章：单线程版本
- 第 21 章：多线程版本（线程池）
- 第 22 章：异步版本（tokio）
```

#### link-short（第 23 章）
```text
项目名称：link-short
定位：短链接服务 REST API
类比：bit.ly / t.cn 的简化版

API 设计：
POST /links
  Body: { "url": "https://example.com/very/long/path" }
  Response: { "code": "abc123", "short_url": "http://localhost:3000/abc123" }

GET /:code
  -> 302 Redirect to original URL

GET /links/:code/stats
  Response: { "code": "abc123", "url": "...", "clicks": 42, "created_at": "..." }

DELETE /links/:code
  -> 204 No Content

使用示例：
# 创建短链接
$ curl -X POST http://localhost:3000/links \
    -H "Content-Type: application/json" \
    -d '{"url": "https://github.com/rust-lang/rust"}'
{"code":"abc123","short_url":"http://localhost:3000/abc123"}

# 访问短链接（会跳转）
$ curl -L http://localhost:3000/abc123

# 查看统计
$ curl http://localhost:3000/links/abc123/stats
{"code":"abc123","clicks":5,"created_at":"2025-01-29T10:00:00Z"}

功能需求：
✓ 创建短链接（自动生成短码）
✓ 访问短链接跳转
✓ 统计点击次数
✓ 删除链接
✓ SQLite 持久化
✓ 链接过期时间（可选）
```

#### api-cli（第 24 章）
```text
项目名称：api-cli
定位：命令行 HTTP/API 客户端
类比：简化版 curl/httpie

使用示例：
# GET 请求
$ api-cli get https://api.example.com/users
[
  {"id": 1, "name": "Alice"},
  {"id": 2, "name": "Bob"}
]

# POST 请求
$ api-cli post http://localhost:3000/links \
    --json '{"url": "https://github.com"}'
{"code": "abc123", ...}

# 自定义 Headers
$ api-cli get https://api.example.com/protected \
    -H "Authorization: Bearer token123"

# 与 link-short 交互
$ api-cli post http://localhost:3000/links --json '{"url": "https://rust-lang.org"}'
$ api-cli get http://localhost:3000/links/abc123/stats

需求清单：
✓ 支持 GET/POST/PUT/DELETE
✓ JSON 请求体
✓ 自定义 Headers
✓ JSON 格式化输出（彩色高亮）
✓ 响应状态码和耗时显示
```

---

## 3. 项目设计详解

### 3.1 项目贯穿线

教程有两条主要的项目线：

**CLI 工具线**（体现 Unix 哲学）：
```
echo-rs(1) → word-count(2-3) → uniq-rs(4-5) → freq(12) → find-rs(16)
```
简单工具，每个独立完整，体现"做好一件事"。

> 说明：CLI 工具线在第 6-11 章中断，此时学习者专注于 task-cli 演进和基础概念学习。
> 在第 6 章开头应提示："CLI 工具线将在第 12 章继续，接下来几章我们专注于构建一个完整应用。"

**Task CLI 演进线**（体现完整应用）：
```
task-cli(6) → +枚举(7) → +文件(8) → +trait(9) → +闭包(11) → +serde(14) → +clap(15)
```
一个项目逐步完善，体验真实项目演进。

> 说明：第 10 章（泛型）和第 12 章（迭代器）使用独立项目。
> 建议在这些章节末尾增加可选练习，将所学应用到 task-cli 上。

### 3.2 各项目设计理由

| 项目 | 章节 | 为什么选择 |
|-----|------|-----------|
| echo-rs | 1 | 最简单的 Unix 工具，5 分钟完成 |
| word-count | 2-3 | 需要字符串处理，自然引出类型和模块 |
| uniq-rs | 4-5 | 需要比较字符串，自然遇到所有权问题 |
| task-cli | 6-9, 11, 14-15 | 贯穿多章，体验真实项目演进 |
| mini-vec | 10 | 泛型概念演示，实现容器加深理解 |
| freq | 12 | 需要 HashMap + 迭代器，经典词频统计 |
| text-toolkit | 17 | I/O 综合应用，Unix 工具组合 |
| find-rs | 16 | 文件系统操作，实用性强 |
| parallel-hash | 18 | 并发处理的自然场景 |
| log-watcher | 19 | channel 的典型应用场景 |
| kv-server | 20-22 | 网络编程的经典入门项目，三版本演进 |
| link-short | 23 | Web 框架实战，REST API 完整案例 |
| api-cli | 24 | HTTP 客户端，可与 link-short 联动测试 |

### 3.3 Task CLI 演进详解

Task CLI 是核心贯穿项目，设计如下：

**第 6 章（基础版）**：
```rust
struct Task {
    id: u32,
    title: String,
    done: bool,
}
```
- 只有内存存储
- 学习 struct 和 impl

**第 7 章（+枚举）**：
```rust
enum Status { Pending, InProgress, Done }
struct Task {
    id: u32,
    title: String,
    status: Status,
    due: Option<String>,  // 引入 Option
}
```

**第 8 章（+错误处理+文件）**：
- 保存到纯文本文件
- 学习 Result 和 ? 运算符

**第 9 章（+Trait）**：
- 实现 Display 和 Debug
- 漂亮的输出格式

**第 11 章（+闭包）**：
- 添加 --filter 功能
- 使用闭包实现灵活过滤

**第 14 章（+Serde）**：
- JSON 存储任务数据
- TOML 配置文件

**第 15 章（+Clap）**：
- 完整的子命令
- 帮助信息
- 配置目录管理

---

## 4. 闭包章节设计

### 4.1 为什么新增闭包章节

审查发现：第 12 章迭代器大量使用闭包（如 `.filter(|x| ...)`），但之前没有专门讲解。

闭包是 Rust 的重要特性，且与所有权系统紧密相关，值得单独讲解。

### 4.2 章节定位

- 位于第 10 章（泛型）之后、第 12 章（迭代器）之前
- 作为第 11 章
- 为迭代器的 map/filter/fold 等方法做准备

### 4.3 核心内容

1. **闭包语法**：`|args| expression`
2. **捕获模式**：借用、可变借用、移动
3. **三种 trait**：Fn、FnMut、FnOnce
4. **与 Java Lambda 对比**：捕获语义差异

### 4.4 项目设计

为 task-cli 添加过滤功能：
```rust
// 使用闭包实现灵活过滤
tasks.iter()
    .filter(|t| t.status == Status::Pending)
    .filter(|t| t.priority == Priority::High)
```

---

## 5. Serde 专题设计

### 5.1 为什么 Serde 如此重要

Serde 是 Rust 生态设计哲学的最佳体现：

**Unix 哲学映射**：
- "做好一件事" → serde 只做序列化框架
- "程序间协作" → serde_json, toml 等格式 crate 独立
- "组合优于集成" → 自由组合不同格式

**关注点分离**：
```text
数据结构定义 (你的代码)
       ↓ derive
Serialize/Deserialize trait (serde)
       ↓ 组合
具体格式实现 (serde_json / toml / ...)
```

### 5.2 教学重点

1. **使用者视角**：
   - 如何添加 derive
   - 如何处理可选字段、重命名
   - 不深入宏实现

2. **生态哲学**：
   - 为什么 serde 不内置 JSON 支持？
   - 这种设计带来什么好处？
   - Rust 生态的普遍模式

3. **实践应用**：
   - JSON 用于数据交换
   - TOML 用于配置文件
   - 体验组合的威力

---

## 6. 所有权教学策略

### 6.1 教学难点分析

所有权是 Java 开发者学 Rust 的最大障碍：
- Java：所有对象都在堆上，GC 管理
- Rust：值有明确的所有者，编译期检查

### 6.2 教学策略

**第 4 章：建立基本概念**
- 用 uniq-rs 自然遇到"值被移动"
- 先理解问题，再给出解决方案（借用）
- 类比：借东西 vs 送东西

**第 5 章：深入借用规则**
- 可变借用的限制
- 从"防止数据竞争"角度解释规则
- 生命周期只讲概念和省略规则，不讲显式标注

**后续章节：自然强化**
- 在实际项目中不断运用
- 错误时回顾所有权规则
- 形成肌肉记忆

### 6.3 与 Java 对比框架

```text
Java:                           Rust:
Object obj = new Object();      let obj = Object::new();
// obj 是引用                    // obj 拥有值

Object obj2 = obj;              let obj2 = obj;
// obj 和 obj2 指向同一对象      // obj 不再有效！值移动到 obj2

// GC 跟踪引用计数              // 编译器跟踪所有权
```

---

## 7. 模块化教学设计

### 7.1 为什么第 3 章就引入

传统 Rust 教程模块讲得晚，但这有问题：
- 学习者养成全写 main.rs 的习惯
- 后期改习惯成本高
- 不符合真实项目实践

### 7.2 渐进式引入

**第 3 章：基础模块**
```text
src/
├── main.rs
└── counter.rs    // mod counter;
```
- 单文件模块
- pub 可见性

**第 6 章：模块目录**
```text
src/
├── main.rs
├── task/
│   ├── mod.rs
│   └── storage.rs
└── cli.rs
```
- 目录模块
- 模块层级

**第 15 章：完整项目结构**
```text
src/
├── main.rs
├── lib.rs        // library crate
├── cli/
├── task/
└── config/
```
- lib.rs 与 main.rs 分离
- 可测试的库设计

---

## 8. 与 Java 对比策略

### 8.1 对比点列表

| 主题 | Java | Rust | 设计意图差异 |
|-----|------|------|-------------|
| 空值 | null | Option<T> | 类型系统消除空指针 |
| 错误 | Exception | Result<T, E> | 显式错误处理 |
| 接口 | interface | trait | 可为外部类型实现 |
| 泛型 | 类型擦除 | 单态化 | 零成本抽象 |
| 继承 | class extends | 组合 + trait | 组合优于继承 |
| 并发 | synchronized | 所有权系统 | 编译期检查 |
| 闭包 | Lambda | Closure | 所有权捕获 |
| 构建 | Maven/Gradle | Cargo | 更简洁统一 |

### 8.2 对比原则

1. **不贬低 Java**：各有设计取舍
2. **强调意图**：Rust 为什么这样选择
3. **心智模型**：帮助建立新的思维方式

---

## 9. 扩展点规划

### 9.1 可选扩展章节

核心章节（1-23 章）之后，可按需扩展：

**Web 方向**：
- 更复杂的 Axum 应用
- 数据库操作进阶（SQLx）

**系统方向**：
- 跨平台开发
- FFI 调用 C 库

**工具方向**：
- 更复杂的 CLI（TUI）
- 性能分析和优化

### 9.2 扩展原则

- 核心章节完成后才扩展
- 每个扩展独立成章
- 不影响核心学习路径

---

## 10. 章节规模平衡策略

### 10.1 高密度章节处理

以下章节内容密度较高，已采取措施：

| 章节 | 原问题 | 处理策略 |
|-----|--------|---------|
| 第 12 章 | Vec + HashMap + 迭代器 | HashMap 随 freq 项目按需引入 |
| 第 17 章 | 4 个子工具 | 2 个核心必做 + 2 个扩展练习 |
| 第 18 章 | 线程 + Arc + Mutex + 消息传递 | 消息传递移至第 19 章 |
| 第 23 章 | 概念密集 | 核心路径优先，高级特性作为"进阶阅读" |

### 10.2 规模渐进原则

```text
第 1-5 章：小规模（每章 ~1500 字 + 50 行代码）
第 6-11 章：中规模（每章 ~2500 字 + 100 行代码）
第 12-17 章：中大规模（每章 ~3000 字 + 150 行代码）
第 18-23 章：大规模（每章 ~4000 字 + 200 行代码）
```

这符合学习者能力增长曲线。

---

## 11. 实施检查清单

### 每章编写前检查

- [ ] 本章核心概念是否不超过 3 个？
- [ ] 项目是否实用且有趣？
- [ ] 是否有需要用到的前置知识没讲？
- [ ] 是否引入了暂时用不到的知识？
- [ ] 章节规模是否符合渐进原则？
- [ ] 本章涉及哪些需要解释的命名/API？（提前列出清单）

### 每章编写后检查

- [ ] 代码是否可以编译运行？
- [ ] 是否解释了设计意图？
- [ ] 与 Java 的对比是否准确？
- [ ] 练习难度是否适中？
- [ ] 是否包含了最佳实践部分？
- [ ] 代码示例是否遵循了命名解释原则？

### 项目完整性检查

- [ ] Cargo.toml 依赖版本是否最新稳定？
- [ ] 代码风格是否符合 rustfmt？
- [ ] 是否有必要的错误处理？
- [ ] README 是否清晰？

---

## 12. 版本管理

### 12.1 依赖版本策略

- 使用 `依赖 = "最新稳定版"` 格式
- 定期更新检查
- 重大变更时更新文档

### 12.2 Rust 版本

- 最低支持：1.75+（推荐 1.80+）
- Edition：2021
- 推荐使用 rustup 管理

---

## 13. 命名与 API 解释策略

### 13.1 解释原则

**通过理解记忆，而非通过记忆使用**：

每当引入新的：
- 标准库模块（如 `std::io`、`std::collections`）
- 常用函数（如 `collect`、`unwrap`、`map`）
- 类型名称（如 `Vec`、`HashMap`、`BufReader`）

都要解释其命名由来和设计意图。

### 13.2 常见前缀/后缀速查

在教程中逐步介绍这些命名模式：

| 模式 | 含义 | 示例 |
|-----|------|------|
| `try_*` | 可能失败，返回 Result | `try_from` |
| `*_or` | 带默认值版本 | `unwrap_or` |
| `*_mut` | 可变版本 | `iter_mut` |
| `as_*` | 低成本转换（借用） | `as_str` |
| `to_*` | 可能有开销的转换 | `to_string` |
| `into_*` | 消耗所有权的转换 | `into_iter` |
| `is_*` | 布尔判断 | `is_empty` |
| `*_unchecked` | 不安全版本 | `get_unchecked` |

---

## 14. I/O 与文本处理专题

### 14.1 设计理由

用户反馈需要更多实用的 I/O 项目，特别是：
- 批量文件读取
- 文件写入
- 文本统计

这是非常实用的技能，在日常工作中经常用到。

### 14.2 text-toolkit 设计

将多个小工具组合成一个工具集，体现：
- Unix 哲学：每个工具做好一件事
- 代码复用：共享的文件处理逻辑
- 项目组织：workspace 或模块分离

### 14.3 学习要点

| 工具 | 主要学习点 |
|-----|-----------|
| batch-rename | 目录遍历、预览模式设计 |
| line-stats | 流式处理、统计聚合 |
| text-replace | 正则表达式、原子写入 |
| csv-query | 结构化数据处理 |

---

## 15. Workspace 引入策略

### 15.1 引入时机

采用渐进式引入，在第 13 章正式讲解：
- 早期（1-12章）：独立项目，简单清晰
- 第 13 章：正式引入 workspace
- 后续（14+）：在 workspace 中组织项目

### 15.2 为什么在第 13 章

- 此时已有足够多的项目，自然需要统一管理
- 为后续 text-toolkit（多工具组合）打基础
- 不会太早增加认知负担，也不会太晚错过实践机会

### 15.3 Workspace 内容规划

**基础内容**：
- workspace 配置语法
- `[workspace.dependencies]` 共享依赖
- 成员间 path 依赖
- workspace 命令

**最佳实践**：
- 何时用 workspace vs 单独项目
- 目录结构约定
- 版本管理策略

---

## 16. 最佳实践教学策略

### 16.1 设计理由

用户明确要求：不仅要讲"怎么用"，还要讲"怎么用好"。

### 16.2 每章应包含

| 部分 | 内容 |
|-----|------|
| 何时使用 | 适用场景 |
| 何时避免 | 误用场景 |
| 常见陷阱 | 新手易错点 |
| 惯用写法 | idiomatic Rust |

### 16.3 关键章节的最佳实践清单

| 章节 | 最佳实践主题 |
|-----|-------------|
| 所有权 | clone vs 借用的选择 |
| 错误处理 | unwrap / ? / match 的选择 |
| 闭包 | 捕获模式选择、move 关键字 |
| 迭代器 | iter 三兄弟的选择 |
| 模块 | pub 最小化原则 |
| Workspace | 项目拆分策略 |
| Serde | 字段重命名、默认值处理 |
| 并发 | Arc vs Rc、Mutex vs RwLock |

---

## 17. Web 框架教学设计

### 17.1 框架选择：Axum

**为什么选择 Axum 而不是其他**：

| 框架 | 优点 | 缺点 | 选择理由 |
|-----|------|------|---------|
| Axum | tokio 官方，设计优雅 | 较新 | ✅ 选择 |
| Actix-web | 性能最强 | 学习曲线陡 | 过于复杂 |
| Rocket | 易用 | 稳定性历史问题 | 不推荐 |
| Warp | 函数式 | 风格独特 | 不够主流 |

### 17.2 项目选择：短链接服务

**为什么选择这个项目**：
- 功能清晰：CRUD + 统计，覆盖常见场景
- 数据库集成：自然引入 SQLx
- 有趣实用：可以实际部署使用
- 易于扩展：可添加认证、限流等

### 17.3 教学重点

1. **项目结构**：如何组织 Web 项目
2. **路由设计**：RESTful API 设计
3. **状态管理**：共享状态的正确方式
4. **错误处理**：统一错误响应
5. **与 Spring 对比**：帮助 Java 开发者理解差异

### 17.4 最佳实践清单

- 使用 `State` 管理应用状态
- 使用 `Extension` 传递配置
- 错误类型实现 `IntoResponse`
- 使用 `tracing` 而非 `log`
- 优雅关闭处理

---

## 18. 顶级组件深度介绍策略

### 18.1 设计理由

用户指出：对于 Serde 这样的顶级组件，仅讲"怎么用"是不够的。
Serde 的独特之处在于——**其他语言没有类似的设计**。

这反映了更深层的教学需求：理解「为什么 Rust 能做到」。

### 18.2 深度介绍框架

每个顶级组件需要介绍：

```text
1. 背景（Who/Why/History）
2. 设计理念（Core Design）
3. 独特价值（Why Rust Can Do This）
4. 与其他语言对比（Comparison）
5. 实际使用（Usage）
6. 最佳实践（Best Practices）
```

### 18.3 重点组件列表

| 组件 | 章节 | 深度介绍重点 |
|-----|------|-------------|
| Serde | 14 | 框架与格式分离的独特设计 |
| Clap | 15 | derive 宏的声明式设计 |
| Tokio | 22 | 异步运行时的工作原理 |
| Axum | 23 | Extractor 模式 vs Spring DI |
| SQLx | 23 | 编译时 SQL 检查的实现原理 |

### 18.4 Serde 为什么独特

**Java (Jackson)**：
```java
ObjectMapper mapper = new ObjectMapper();
String json = mapper.writeValueAsString(obj);
// Jackson = JSON 绑定，要 YAML 得换库
```

**Rust (Serde)**:
```rust
// serde 不绑定格式！
#[derive(Serialize)]
struct User { name: String }

// 可以输出任何格式
serde_json::to_string(&user)?;  // JSON
toml::to_string(&user)?;        // TOML
serde_yaml::to_string(&user)?;  // YAML
```

这是 Rust trait 系统 + 过程宏 + 零成本抽象的完美体现。

---

## 19. 待决定事项

在实施过程中如遇到设计决策需要用户确认，记录在此：

| 问题 | 选项 | 决定 | 日期 |
|-----|------|------|------|
| Task CLI 贯穿设计 | 贯穿/部分/独立 | 贯穿 | 2025-01 |
| 命名解释原则 | 添加 | 已添加 | 2025-01 |
| I/O 文本处理 | 添加专题章节 | 已添加第 17 章 | 2025-01 |
| Workspace 引入 | 渐进式 | 第 13 章引入 | 2025-01 |
| 最佳实践 | 每章必须包含 | 已添加到规范 | 2025-01 |
| Web 框架 | Axum/Actix/Rocket | Axum，第 23 章 | 2025-01 |
| Web 项目 | 短链接服务 | link-short | 2025-01 |
| 顶级组件介绍 | 深度介绍 | 背景+理念+独特价值 | 2025-01 |
| 项目需求分析 | 每个项目必须有 | 功能概览+需求+效果演示 | 2025-01 |
| 章节编号 | 去除 .5 | 重新编号为连续整数 | 2025-01 |
| 闭包章节 | 新增 | 第 11 章 | 2025-01 |
