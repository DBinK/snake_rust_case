# snake_rust_case

Learn Rust by a terminal snake game. 面向会 C/Python、零基础 Rust 的读者：
一个 ~330 行的终端贪吃蛇 + 逐行对照的 Python 实现，聚焦 Rust 与 C/Python 的**理念差异**。

## 结构（Rust 惯例布局）

```
src/           游戏本体：main.rs（循环）/ game.rs（纯逻辑+测试）/ terminal.rs（I/O）
examples/      minimal.rs  第 03 章最小可跑版：方向键移动一个字符（56 行）
doc/           章节文档，从 01 开始按顺序读
python/        snake.py    Python 对照版（标准库 curses，可直接跑）
```

## 快速跑起来

```bash
cargo run                          # 完整版游戏
cargo run --example minimal        # 第 03 章最小 demo
cargo test                         # 纯逻辑单测
python3 python/snake.py            # Python 对照版（需真终端）
```

玩法：开局停在 Ready 屏，按任意方向键起步；h/j/k/l 或方向键转向；
撞墙/咬到自己 → 结算屏按 r 重开、q 退出。

## 各章对应代码

| 章 | 文档 | 代码 |
|---|---|---|
| 03 最小可跑版 | `doc/03_最小可跑版.md` | `examples/minimal.rs` |
| 04 完整实现 | `doc/04_完整实现.md` | `src/` |
| 05 Python 对照 | `doc/05_Python对照.md` | `python/snake.py` |
