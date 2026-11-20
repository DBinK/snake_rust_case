#!/usr/bin/env python3
"""贪吃蛇 —— Rust 版（src/ 下的根 crate）的 Python 对照实现。

刻意与 Rust 版同构，方便逐段对照：
  Game      → game.rs::Game      （纯逻辑，不碰终端）
  Phase     → game.rs::Phase     （Ready/Playing/Over 三态）
  term()    → terminal.rs::RawMode（curses.wrapper 相当于 RAII Drop）
  read_input→ terminal.rs::read_input
  draw      → terminal.rs::draw
  main 循环 → main.rs

对照要点都标注在行尾注释里：# ← Rust: ...
跑法：python3 snake.py   （macOS/Linux 直接用自带 curses；Windows 需 pip install windows-curses）
"""

import curses
import random
from collections import deque
from enum import Enum, auto

WIDTH, HEIGHT = 20, 12
TICK_START, TICK_MIN, TICK_STEP = 0.20, 0.06, 0.01  # 起步 200ms，每分 -10ms，下限 60ms


def tick_for(score):
    # ← Rust: saturating_sub(...).max(MIN_TICK_MS)
    return max(TICK_MIN, TICK_START - score * TICK_STEP)


class Dir(Enum):
    """方向。

    ← Rust: `enum Dir` 是 C 式枚举 + 编译期 match 穷尽检查。
    Python 的 Enum 只是类，漏写一个分支编译器不会拦你，
    要靠运行时 KeyError 才发现 —— "值域在类型里" vs "值域在约定里"。
    """

    UP, DOWN, LEFT, RIGHT = auto(), auto(), auto(), auto()

    def opposite(self):
        return {
            Dir.UP: Dir.DOWN,
            Dir.DOWN: Dir.UP,
            Dir.LEFT: Dir.RIGHT,
            Dir.RIGHT: Dir.LEFT,
        }[self]  # ← Rust: match self { ... }，漏 arm 会编译失败


class Phase(Enum):
    """游戏阶段。Python 惯例：两个 bool（started/over）也能表达，
    但 (started=False, over=True) 这种非法组合没人拦你；
    enum 至少让状态只有一个字段。Rust 版是类型级约束。"""

    READY, PLAYING, OVER = auto(), auto(), auto()


class Pos(tuple):
    """坐标点 (x, y)。Python 惯例直接上 tuple/namedtuple；
    单独定义只为对照 Rust 的 `struct Pos { x, y }`。"""

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
    """纯逻辑层。和 Rust 版的差别：没有借用检查，
    `body` 字段谁都能改，'不对外暴露可写引用'全靠自觉。"""

    def __init__(self):
        self.body = deque()
        self.dir = Dir.RIGHT
        self.reset()

    def reset(self):
        """开局/重开。Ready 阶段蛇停着不动，等第一个方向键。"""
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
            self.phase = Phase.PLAYING  # 第一个方向键 = 起步方向
        elif self.phase is Phase.PLAYING and d != self.dir.opposite():
            self.dir = d

    def _next_head(self):
        h = self.head()
        if self.dir is Dir.UP:
            return Pos(h.x, max(0, h.y - 1))  # saturating_sub 手写版
        if self.dir is Dir.DOWN:
            return Pos(h.x, h.y + 1)
        if self.dir is Dir.LEFT:
            return Pos(max(0, h.x - 1), h.y)
        return Pos(h.x + 1, h.y)


    def step(self):
        """推进一步。Rust 版这里被借用检查器盯着；
        Python 版同样的操作序列，没人拦你，但也更容易写错序。"""
        if self.phase is not Phase.PLAYING:
            return  # ← Rust: if self.phase != Phase::Playing { return; }
        nxt = self._next_head()
        hit_wall = not (0 <= nxt.x < WIDTH and 0 <= nxt.y < HEIGHT)
        will_grow = nxt == self.food
        # 尾巴即将让位，咬身判定要先去掉尾格
        body = list(self.body) if will_grow else list(self.body)[:-1]
        # nxt == head 意味着被 max(0,...) 钳在边界原地踏步 = 撞墙；
        # 不能靠咬身判：1 节蛇时尾格已被去掉，原地踏步会漏判成"卡住不死"。
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
        # 从空格集合均匀抽样 —— ← Rust: free.choose(&mut rand::rng())
        free = [
            Pos(x, y)
            for y in range(HEIGHT) for x in range(WIDTH)
            if Pos(x, y) not in self.body
        ]
        return random.choice(free)


def read_input(stdscr, timeout):
    """非阻塞读输入。返回 ('move', Dir) / ('quit',) / ('retry',) / None。

    ← Rust: Option<Input> enum 把四种结果收进一个类型，
    match 强制处理每个变体。这里用 tuple 惯例，忘了判
    'quit'/'retry' 的 bug 只能运行时踩出来。
    """
    stdscr.timeout(int(timeout * 1000))
    k = stdscr.getch()
    table = {
        curses.KEY_UP: Dir.UP, ord("k"): Dir.UP,
        curses.KEY_DOWN: Dir.DOWN, ord("j"): Dir.DOWN,
        curses.KEY_LEFT: Dir.LEFT, ord("h"): Dir.LEFT,
        curses.KEY_RIGHT: Dir.RIGHT, ord("l"): Dir.RIGHT,
    }
    if k in table:
        return ("move", table[k])
    if k in (ord("q"), 27):  # q / ESC
        return ("quit",)
    if k in (ord("r"), curses.KEY_ENTER, 10, 13):
        return ("retry",)
    return None


def draw(stdscr, g):
    stdscr.erase()
    stdscr.addstr(0, 0, f"  Score: {g.score}   (h/j/k/l or arrows, q to quit)")
    for y in range(HEIGHT):
        row = "  " + "".join(
            "O" if Pos(x, y) in g.body
            else "*" if Pos(x, y) == g.food
            else "."
            for x in range(WIDTH)
        )
        stdscr.addstr(y + 1, 0, row)
    if g.phase is Phase.READY:
        stdscr.addstr(HEIGHT + 1, 0, "  Ready — press an arrow key to start, q to quit")
    elif g.phase is Phase.OVER:
        stdscr.addstr(HEIGHT + 1, 0, f"  Game Over! score={g.score}   r: retry   q: quit")
    stdscr.refresh()


def term(stdscr):
    """curses.wrapper 替你做了 Rust 里 RawMode/Drop 的事：
    进入时初始化 + 结束时恢复终端 —— 作用域资源管理，两边思路一致。"""
    g = Game()
    while True:
        draw(stdscr, g)
        if g.phase is Phase.READY:
            # 就绪：只等按键，蛇不动
            match read_input(stdscr, 3600):
                case ("quit",):
                    break
                case ("move", d):
                    g.turn(d)  # 起步即转向
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
        # ← Rust: match g.phase { Ready => ..., Playing => ..., Over => ... }
        # 三种形态同一种结构，但 Rust 的 match 穷尽检查保证你三个 phase 都写了分支


if __name__ == "__main__":
    curses.wrapper(term)
