# Snake Rust Case

面向具备 C 或 Python 背景、零基础学习 Rust 的开发者。

本项目通过一个约 300 行的终端贪吃蛇游戏，配合结构完全同构的 Python 实现，系统讲解 Rust 的核心编程范式与语言思维。教程聚焦所有权、借用检查、代数数据类型、RAII 资源管理与类型驱动设计，帮助读者建立对 Rust 底层心智模型的清晰认知。

## 教程路线

建议按照编号顺序依次阅读文档：

| 章节 | 文档 | 对应代码 | 核心教学目标 |
|---|---|---|---|
| 01 需求规范 | [doc/01_需求.md](file:///Users/binn/it/snake_rust_case/doc/01_需求.md) | - | 厘清领域模型、时序要求与非目标边界 |
| 02 架构设计 | [doc/02_设计.md](file:///Users/binn/it/snake_rust_case/doc/02_设计.md) | - | 分层架构与类型驱动设计，非法状态无法在类型中表达 |
| 03 最小可跑版 | [doc/03_最小可跑版.md](file:///Users/binn/it/snake_rust_case/doc/03_最小可跑版.md) | [examples/minimal.rs](file:///Users/binn/it/snake_rust_case/examples/minimal.rs) | 终端 Raw Mode、非阻塞输入与 Rust 2024 `let-else` 语法 |
| 04 完整实现 | [doc/04_完整实现.md](file:///Users/binn/it/snake_rust_case/doc/04_完整实现.md) | [src/](file:///Users/binn/it/snake_rust_case/src/) | 状态机驱动、借用检查机制、无偏随机采样与单测 |
| 05 语言心智对比 | [doc/05_Python对照.md](file:///Users/binn/it/snake_rust_case/doc/05_Python对照.md) | [python/snake.py](file:///Users/binn/it/snake_rust_case/python/snake.py) | 编译期约束与运行时约定的深层对比 |

## 项目结构

```
src/
  ├── main.rs       程序入口与状态机驱动循环
  ├── game.rs       纯领域逻辑：状态、移动、碰撞与单元测试
  └── terminal.rs   终端适配层：Raw Mode 守卫、事件捕获与双缓冲绘制
examples/
  └── minimal.rs    第 03 章最小原型（单字符终端控制）
doc/                完整教程文档
python/
  └── snake.py      与 Rust 版同构的 Python 参考实现
```

## 快速开始

### 运行环境要求

- Rust 1.85+（支持 Rust 2024 Edition）
- Python 3.10+（用于运行对照版本，类 Unix 系统使用标准库 `curses`）

### 运行命令

```bash
# 运行完整 Rust 版游戏
cargo run

# 运行第 03 章最小终端原型
cargo run --example minimal

# 执行领域逻辑单元测试
cargo test

# 运行 Python 对照版
python3 python/snake.py
```

### 操作指南

- 起步：游戏初始处于 Ready 状态，按下任意方向键或 `h`/`j`/`k`/`l` 起步。
- 移动：使用方向键或 Vim 键位（`h` 左、`j` 下、`k` 上、`l` 右）调整蛇的行进方向。
- 结算：碰撞边界或自身后进入结算状态，按 `r` 或回车重新开始，按 `q` 或 `Esc` 退出。
