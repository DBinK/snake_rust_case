# 05 Python 对照：语言哲学与心智模型

在 `python/snake.py` 中，我们提供了一份结构与 Rust 版完全同构的 Python 参考实现（约 215 行）。二者在类名、方法名、模块划分与算法细节上均一一对应。

通过这份对照代码，读者可以直观洞察两种编程语言在深层哲学上的本质差异：**“运行时的动态约定”与“编译期的严格证明”**。

## 核心设计对照一览

| 关注维度 | Rust（`src/`） | Python（`python/snake.py`） | 深层设计差异 |
|---|---|---|---|
| 方向枚举 | `enum Dir`，编译器强制穷尽检查 | `Enum` 类，通过字典映射 | Rust 遗漏分支在编译期直接报错，Python 漏判分支在运行时抛出 `KeyError` |
| 空值语义 | `Option<Input>`，强制解包 | 返回 `None`，调用方容易遗漏判断 | Rust 将缺失性编码进类型签名，Python 依赖开发者自觉与文档约定 |
| 控制事件 | `Input::Quit` 强类型变体携带数据 | `("quit",)` 元组惯例约定 | Rust 消除拼写与元组结构不一致隐患，Python 仅靠惯例维护 |
| 集合操作 | `VecDeque<Pos>` 结合借用检查 | `deque` 随意读写 | Rust 阻止在迭代遍历的同时修改底层容器，Python 依赖手工切片或易发迭代器失效 |
| 资源清理 | `RawMode` 结构体实现 `Drop` 特征 | `curses.wrapper` 上下文管理 | 两者在生命周期作用域理念上一致，Rust 将其沉淀为原生的语言级特性 |
| 坐标界限 | `u16` + `saturating_sub` | `int` + `max(0, ...)` 手动约束 | Rust 从类型级别消除负数，Python 在运行时通过数值计算弥补 |
| 错误传递 | `io::Result<()>` 与 `?` 操作符 | 异常机制与 `try/except` | `?` 强制开发者在接口边界对错误显式分流，异常容易被无意识忽略或意外截断 |
| 领域约束 | 状态机 `Phase` 枚举消除非法状态 | `Phase(Enum)`，但变量可任意赋值 | Rust 限制了非法组合的构造可能，Python 需要靠防御性代码防范脏状态 |

## 核心步进函数逐行对比

以核心的步进函数 `step()` 为例，对比两者的代码实现：

### Rust 实现（`src/game.rs`）

```rust
pub fn step(&mut self) {
    if self.phase != Phase::Playing {
        return;
    }

    let next = self.next_head();
    let hit_wall = next.x >= WIDTH || next.y >= HEIGHT;
    let will_grow = next == self.food;

    let mut body = self.body.iter().copied();
    if !will_grow {
        body.next_back();
    }
    let hit_self = next == self.head() || body.any(|p| p == next);

    if hit_wall || hit_self {
        self.phase = Phase::Over;
        return;
    }

    self.body.push_front(next);
    if will_grow {
        self.score += 1;
        self.place_food();
    } else {
        self.body.pop_back();
    }
}
```

### Python 实现（`python/snake.py`）

```python
def step(self):
    if self.phase is not Phase.PLAYING:
        return

    nxt = self._next_head()
    hit_wall = not (0 <= nxt.x < WIDTH and 0 <= nxt.y < HEIGHT)
    will_grow = nxt == self.food

    body = list(self.body) if will_grow else list(self.body)[:-1]
    hit_self = nxt == self.head() or nxt in body

    if hit_wall or hit_self:
        self.phase = Phase.OVER
        return

    self.body.appendleft(nxt)
    if will_grow:
        self.score += 1
        self.food = self._place_food()
    else:
        self.body.pop()
```

## 三个关键维度的深度分析

### 1. 越界判断与数值类型空间

- **Python**：使用动态大小的有符号整数 `int`。计算出的新坐标可能向负方向越界，也可能向正方向越界，因此判断必须写成双侧区间：`0 <= nxt.x < WIDTH`。
- **Rust**：显式使用 `u16` 无符号整数。在调用 `next_head()` 计算时，向下或向左使用 `saturating_sub(1)` 饱和减法，保证数值下限始终为 0，不可能出现负数。因此越界判断只需单向判定：`next.x >= WIDTH || next.y >= HEIGHT`。数值类型自身替代码承担了一半的校验责任。

### 2. 内存模型与零拷贝视图

- **Python**：为了在判定咬自身时不把即将移出的尾部算进去，Python 必须显式创建一个新的列表对象：`list(self.body)[:-1]`。这涉及新容器的内存分配与全量元素浅拷贝。
- **Rust**：通过 `self.body.iter().copied()` 创建一个栈上的轻量双端迭代器。调用 `body.next_back()` 仅是调整了迭代器的内部游标，**全过程零内存分配、零容器拷贝**。而能够安全做到这一点的基础，正是借用检查器确认了在迭代器活跃期间，没有代码能并发修改底层的 `self.body`。

### 3. 类型安全与心智负担的转移

在初学 Rust 时，许多开发者会感觉编译器“过于挑剔”，编写代码不如在 Python 中随心所欲。

但通过本项目的完整实践可以发现：
- Python 的自由度建立在把检查推迟到运行时的代价之上。开发者必须依赖严谨的自律、海量的断言与单元测试，来防范漏掉的分支、拼错的键名与非法状态组合。
- Rust 将这些潜在隐患全部前置到编译阶段。看似在编码初期付出了更多类型定义与借用梳理的心智成本，但一旦通过编译，程序即可获得极高的确定性与运行期稳定性。

编译器不再是找茬的对手，而是为你 7x24 小时审查代码隐患的资深助手。
