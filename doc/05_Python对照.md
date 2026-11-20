# 05 Python 对照：同一个游戏，两种语言观

`python/snake.py`（215 行）与 `src/` 刻意同构：
类名、方法名、分层完全对齐，逐行可以对照着看。

## 对照表

| 场景 | Rust（`src/`） | Python（`python/snake.py`） | 差异的本质 |
|---|---|---|---|
| 方向 | `enum Dir`，`match` 穷尽 | `Enum` 类，`dict.get` 或 `if` 链 | Rust 漏分支编译错；Python 漏了运行时炸 |
| 没按键 | `Option<Input>`，必须 `match` | `getch()` 返回 -1/None，可忽略 | Rust 把"可能没有"写进签名；Python 靠自觉 |
| 退出键 | `Input::Quit` 是类型变体 | `("quit",)` tuple 约定 | 前者编译器查；后者文档查 |
| 蛇身 | `VecDeque<Pos>` + 借用规则 | `deque` + 谁都能改 | Rust 编译期挡"迭代中修改"；Python 运行时炸或静默错 |
| 终端恢复 | `RawMode` + `Drop` | `curses.wrapper` | 同一种资源管理思路，Rust 把它变成语言特性 |
| 坐标越界 | `u16` + `saturating_sub` | `max(0, ...)` 手写 | Rust 用类型表达"非负"；Python 用运行时值表达 |
| 错误 | `io::Result<()>` + `?` | 异常 + `try/except` | `?` 是编译期强制的"要么处理要么传出去"；异常可以静默吞 |
| 步进逻辑 | `step()` 受借用检查约束 | `step()` 无约束 | Rust 让你排对顺序；Python 让你自己对顺序 |

## 一段逐行对照：`step()`

```rust
// Rust
let next = self.next_head();
let hit_wall = next.x >= WIDTH || next.y >= HEIGHT;
let will_grow = next == self.food;
let mut body = self.body.iter().copied();
if !will_grow { body.next_back(); }
let hit_self = body.any(|p| p == next);
if hit_wall || hit_self { self.over = true; return; }
self.body.push_front(next);
```

```python
# Python
nxt = self._next_head()
hit_wall = not (0 <= nxt.x < WIDTH and 0 <= nxt.y < HEIGHT)
will_grow = nxt == self.food
body = list(self.body) if will_grow else list(self.body)[:-1]
hit_self = nxt in body
if hit_wall or hit_self:
    self.over = True
    return
self.body.appendleft(nxt)
```

肉眼可见的三处差异：

1. **越界表达**：Rust 的 `u16` 只允许 `>= WIDTH` 这一种"坏"；
   Python 的 int 两边都可能坏，所以判断要写成区间。
2. **借用视图 vs 拷贝**：`iter().copied()` 不产生新内存；
   `list(self.body)` 是实打实的拷贝。Rust 代码更短，但前提是
   编译器确认了"迭代期间没人改它"——这份确认是编译期给的。
3. **`return` vs 缩进**：`return;` 和 Python 的 `return` 字面等价，
   但 Rust 的 `self.over = true` 是字段写入（pub），Python 是属性赋值，
   语义相同，只是 Rust 的字段是否可写由 `pub`/借用规则决定。

## 结论

- **Rust 多管的**：所有权/借用、穷尽性、无 null、资源归还——
  全是"把运行时的坑挪到编译期"。
- **Python 少管的**：类型协议、可变性协议、资源协议——靠文档和测试兜底。
- 这个教程的目标不是"Rust 更啰嗦"，而是让你看见：
  **啰嗦的那部分买的是编译期证明**，编译器成为你的免费 reviewer。
