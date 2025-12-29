# Learn Rust with Claude

面向 Java 开发者的 Rust 实战教程 | 由 Claude AI 辅助创作

## 项目简介

这是一个**由 Claude AI 辅助创作**的 Rust 学习教程，专为有经验的 Java/后端开发者设计。通过 25 个循序渐进的实战项目，帮助你从零掌握 Rust 语言。

本教程的所有内容——章节结构、代码示例、项目设计——均在 Claude 的协助下完成，展示了 AI 辅助编程教育的可能性。

**核心理念**：项目驱动、成就感导向、知其所以然

## 目标读者

- 有 2+ 年 Java/后端开发经验
- 希望学习系统级编程语言
- 喜欢通过实践项目学习
- 想要理解 Rust 设计哲学，而非死记语法

## 教程特色

| 特色 | 说明 |
|-----|------|
| **项目驱动** | 每章围绕一个可运行的实用项目 |
| **渐进式学习** | 每章只引入 1-3 个核心新概念 |
| **Java 对比** | 关键概念与 Java 对比，加速理解 |
| **知其所以然** | 解释 Rust 的设计意图，不只是语法 |
| **最佳实践** | 每章包含惯用写法和常见陷阱 |

## 章节目录

### 第一部分：基础入门（1-5章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 01 | Hello Cargo | hello-rust | Cargo 工具链 |
| 02 | 变量与类型 | temp-converter | let, mut, 基础类型 |
| 03 | 函数与模块 | calculator | fn, mod, 代码组织 |
| 04 | 所有权（上）| string-stats | 所有权, 移动语义 |
| 05 | 所有权（下）| word-frequency | 借用, 生命周期入门 |

### 第二部分：核心概念（6-10章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 06 | 结构体 | todo-struct | struct, impl, 方法 |
| 07 | 枚举与模式匹配 | expr-parser | enum, match, Option |
| 08 | 错误处理 | file-reader | Result, ?, 错误传播 |
| 09 | Trait | shape-calculator | trait, 多态 |
| 10 | 泛型 | data-container | 泛型函数/结构体 |

### 第三部分：实用技能（11-15章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 11 | 生命周期 | config-parser | 'a, 生命周期标注 |
| 12 | 集合与迭代器 | log-analyzer | Vec, HashMap, Iterator |
| 13 | 闭包 | event-system | 闭包, Fn traits |
| 14 | Cargo 与 crates | multi-crate | workspace, 依赖管理 |
| 15 | 序列化 | json-processor | serde, JSON/TOML |

### 第四部分：命令行应用（16-17章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 16 | 文件 I/O | find-rs | std::fs, Path |
| 17 | 文本工具集 | text-toolkit | Workspace 实战 |

### 第五部分：并发编程（18-19章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 18 | 并发基础 | parallel-hash | thread, Mutex, Arc |
| 19 | 消息传递 | task-queue | channel, mpsc |

### 第六部分：网络编程（20-24章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 20 | 网络基础 | echo-server | TcpListener, 协议 |
| 21 | 多线程服务器 | chat-server | 线程池, 广播 |
| 22 | async 入门 | async-downloader | async/await, Future |
| 23 | Web 框架 | rest-api | Axum, 路由, 中间件 |
| 24 | HTTP 客户端 | api-cli | reqwest, API 调用 |

### 第七部分：综合实战（25章）

| 章节 | 主题 | 项目 | 核心概念 |
|-----|------|-----|---------|
| 25 | 综合项目 | 自选项目 | 全面整合 |

### 附录

| 附录 | 内容 |
|-----|------|
| A | Cargo 进阶（Features, build.rs, 发布） |
| B | 测试（单元测试, 集成测试, 文档测试） |
| C | 常用 Crate 速查表 |
| D | 智能指针（Box, Rc, Arc, RefCell） |

## 快速开始

### 环境准备

1. 安装 Rust：
```bash
# Windows (PowerShell)
winget install Rustlang.Rust.MSVC

# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. 验证安装：
```bash
rustc --version
cargo --version
```

3. 推荐 IDE：VS Code + rust-analyzer 插件

### 使用教程

```bash
# 克隆仓库
git clone https://github.com/your-username/learn_rust_with_claude.git
cd learn_rust_with_claude

# 进入第一章
cd chapters/01-hello-cargo

# 阅读教程
cat README.md  # 或用编辑器打开

# 运行项目
cd project
cargo run
```

### 学习建议

1. **按顺序学习**：章节之间有依赖关系
2. **动手实践**：每章都要运行和修改代码
3. **完成练习**：巩固所学概念
4. **对比思考**：与 Java 对比理解设计差异

## 项目结构

```
learn_rust_with_claude/
├── README.md           # 本文件
├── CLAUDE.md           # AI 辅助教学指南
├── OUTLINE.md          # 详细大纲
├── DESIGN.md           # 设计决策
├── chapters/
│   ├── 01-hello-cargo/
│   │   ├── README.md   # 章节教程
│   │   └── project/    # Cargo 项目
│   │       ├── Cargo.toml
│   │       └── src/
│   ├── 02-variables-types/
│   ├── ...
│   ├── 25-final-project/
│   ├── appendix-a-cargo-advanced/
│   ├── appendix-b-testing/
│   ├── appendix-c-crate-reference/
│   └── appendix-d-smart-pointers/
└── .gitignore
```

## 与其他教程的区别

| 对比项 | 本教程 | The Rust Book | Rust by Example |
|-------|-------|---------------|-----------------|
| 目标读者 | Java 开发者 | 通用 | 通用 |
| 学习方式 | 项目驱动 | 概念驱动 | 示例驱动 |
| 项目完整度 | 完整可运行 | 片段为主 | 片段为主 |
| Java 对比 | 系统性对比 | 无 | 无 |
| 设计哲学 | 深入解释 | 部分解释 | 较少 |

## 推荐学习路径

```
第1周：基础入门（1-5章）
  ↓ 掌握所有权，这是 Rust 的核心
第2周：核心概念（6-10章）
  ↓ 理解 Rust 的类型系统
第3周：实用技能（11-15章）
  ↓ 能写实际有用的程序
第4周：并发与网络（16-24章）
  ↓ 掌握 Rust 的优势领域
第5周：综合项目（25章）
  ↓ 独立完成一个完整项目
```

## 常见问题

**Q: 需要多少 Java 经验？**

A: 建议至少 2 年后端开发经验，熟悉 OOP、泛型、多线程等概念。

**Q: 完成教程需要多久？**

A: 每天 1-2 小时，大约 4-6 周可以完成主要章节。

**Q: 遇到问题怎么办？**

A:
1. 仔细阅读编译器错误信息（Rust 的错误信息很友好）
2. 查阅 [Rust 官方文档](https://doc.rust-lang.org/)
3. 搜索 [Rust 用户论坛](https://users.rust-lang.org/)

## 延伸资源

- [The Rust Programming Language](https://doc.rust-lang.org/book/) - 官方教程
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - 示例学习
- [Rustlings](https://github.com/rust-lang/rustlings) - 小练习
- [Rust 中文社区](https://rustcc.cn/) - 中文资源

## 版本信息

- Rust Edition: 2021
- 最低 Rust 版本: 1.75+
- 最后更新: 2025-01

## License

MIT License
