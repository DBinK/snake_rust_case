#!/usr/bin/env python3
"""贪吃蛇 —— 与 Rust 版（src/）同构的 Python 参考实现。

类名、方法名与模块职责与 Rust 实现严格对齐，用于对照学习语言设计差异：
  Game       → game.rs::Game        （纯领域逻辑，与终端 I/O 解耦）
  Phase      → game.rs::Phase       （Ready / Playing / Over 三态状态机）
  term()     → terminal.rs::RawMode （curses.wrapper 提供作用域级的终端恢复）
  read_input → terminal.rs::read_input
  draw       → terminal.rs::draw
  main 循环  → main.rs

关键差异说明已标注在行尾注释中：# ← Rust: ...
运行方式：python3 python/snake.py （类 Unix 系统直接可用，Windows 需 pip install windows-curses）
"""

import curses
import random
from collections import deque
from enum import Enum, auto

WIDTH, HEIGHT = 20, 12
TICK_START, TICK_MIN, TICK_STEP = 0.20, 0.06, 0.01  # 起步 200ms，每分 -10ms，下限 60ms


def tick_for(score):
    # ← Rust: START_TICK_MS.saturating_sub(score * TICK_STEP).max(MIN_TICK_MS)
    return max(TICK_MIN, TICK_START - score * TICK_STEP)


class Dir(Enum):
    """行进方向枚举。

    ← Rust: `enum Dir` 是代数数据类型，match 时享有编译期穷尽检查。
    Python 枚举属于类对象，分支遗漏只能在运行时通过异常暴露。
    """

    UP, DOWN, LEFT, RIGHT = auto(), auto(), auto(), auto()

    def opposite(self):
        return {
            Dir.UP: Dir.DOWN,
            Dir.DOWN: Dir.UP,
            Dir.LEFT: Dir.RIGHT,
            Dir.RIGHT: Dir.LEFT,
        }[self]  # ← Rust: match 穷尽匹配，漏写变体编译直接拦截


class Phase(Enum):
    """游戏阶段状态机。

    ← Rust: `enum Phase` 从类型系统消除 (started=false, over=true) 等非法状态组合。
    """

    READY, PLAYING, OVER = auto(), auto(), auto()


class Pos(tuple):
    """网格坐标 (x, y)。

    ← Rust: `struct Pos { pub x: u16, pub y: u16 }`，无符号整数从类型级别消除负数。
    """

    __slots__ = ()

    def __new__(cls, x, y):
        return super().__new__(cls, (x, y))

    @property
    def x(self):
        return self[0]

    @property
    def y(self):
        return self[1]


class Game:
    """纯领域逻辑层。

    与 Rust 版对照：Python 缺乏借用检查器，容器成员可被任意外部修改；
    Rust 通过私有字段与只读借用接口保证封装完整性。
    """

    def __init__(self):
        self.body = deque()
        self.dir = Dir.RIGHT
        self.reset()

    def reset(self):
        """重置状态回到 Ready 阶段。"""
        self.body.clear()
        self.body.append(Pos(WIDTH // 2, HEIGHT // 2))
        self.dir = Dir.RIGHT
        self.score = 0
        self.phase = Phase.READY
        self.food = self._place_food()

    def head(self):
        return self.body[0]  # ← Rust: *self.body.front().unwrap()

    def turn(self, d):
        if self.phase is Phase.READY:
            self.dir = d
            self.phase = Phase.PLAYING  # 收到首个方向键即起步
        elif self.phase is Phase.PLAYING and d != self.dir.opposite():
            self.dir = d

    def _next_head(self):
        h = self.head()
        if self.dir is Dir.UP:
            return Pos(h.x, max(0, h.y - 1))  # ← Rust: y.saturating_sub(1)
        if self.dir is Dir.DOWN:
            return Pos(h.x, h.y + 1)
        if self.dir is Dir.LEFT:
            return Pos(max(0, h.x - 1), h.y)
        return Pos(h.x + 1, h.y)

    def step(self):
        """推进一步世界物理演进。

        ← Rust: 此处需满足借用检查规则，先只读借用计算碰撞，最后再可变修改容器。
        """
        if self.phase is not Phase.PLAYING:
            return  # ← Rust: if self.phase != Phase::Playing { return; }

        nxt = self._next_head()
        hit_wall = not (0 <= nxt.x < WIDTH and 0 <= nxt.y < HEIGHT)
        will_grow = nxt == self.food

        # 尾节点即将释放，咬身判定需排除尾节点
        # ← Rust: self.body.iter().copied() 零拷贝迭代视图，body.next_back()
        body = list(self.body) if will_grow else list(self.body)[:-1]

        # nxt == head 对应贴边向外移动被钳制在原地的撞墙情况
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

    def _place_food(self):
        # 收集剩余空格无偏采样 —— ← Rust: free.choose(&mut rand::rng())
        free = [
            Pos(x, y)
            for y in range(HEIGHT)
            for x in range(WIDTH)
            if Pos(x, y) not in self.body
        ]
        return random.choice(free)


def read_input(stdscr, timeout):
    """非阻塞输入轮询。

    ← Rust: 返回 Option<Input>，利用代数数据类型强制调用者必须显式处理每个变体。
    """
    stdscr.timeout(int(timeout * 1000))
    k = stdscr.getch()
    table = {
        curses.KEY_UP: Dir.UP,
        ord("k"): Dir.UP,
        curses.KEY_DOWN: Dir.DOWN,
        ord("j"): Dir.DOWN,
        curses.KEY_LEFT: Dir.LEFT,
        ord("h"): Dir.LEFT,
        curses.KEY_RIGHT: Dir.RIGHT,
        ord("l"): Dir.RIGHT,
    }
    if k in table:
        return ("move", table[k])
    if k in (ord("q"), 27):  # q / Esc
        return ("quit",)
    if k in (ord("r"), curses.KEY_ENTER, 10, 13):
        return ("retry",)
    return None


def draw(stdscr, g):
    stdscr.erase()
    stdscr.addstr(0, 0, f"  Score: {g.score}   (h/j/k/l or arrows, q to quit)")
    for y in range(HEIGHT):
        row = "  " + "".join(
            "O"
            if Pos(x, y) in g.body
            else "*"
            if Pos(x, y) == g.food
            else "."
            for x in range(WIDTH)
        )
        stdscr.addstr(y + 1, 0, row)
    if g.phase is Phase.READY:
        stdscr.addstr(
            HEIGHT + 1, 0, "  Ready — press an arrow key to start, q to quit"
        )
    elif g.phase is Phase.OVER:
        stdscr.addstr(
            HEIGHT + 1, 0, f"  Game Over! score={g.score}   r: retry   q: quit"
        )
    stdscr.refresh()


def term(stdscr):
    """curses.wrapper 提供类似 Rust RawMode Drop 的确定性作用域恢复。"""
    g = Game()
    while True:
        draw(stdscr, g)
        if g.phase is Phase.READY:
            match read_input(stdscr, 3600):
                case ("quit",):
                    break
                case ("move", d):
                    g.turn(d)
        elif g.phase is Phase.PLAYING:
            match read_input(stdscr, tick_for(g.score)):
                case ("quit",):
                    break
                case ("move", d):
                    g.turn(d)
            g.step()
        else:  # OVER
            match read_input(stdscr, 3600):
                case ("quit",):
                    break
                case ("retry",):
                    g.reset()


if __name__ == "__main__":
    curses.wrapper(term)
