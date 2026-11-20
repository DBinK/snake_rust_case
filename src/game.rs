//! 纯游戏逻辑：不碰终端、不碰 I/O，因此可以单元测试。
//! 这是刻意的分层 —— 对照 rtd 的 model/service/storage，
//! 我们把"规则"和"展示"分开，borrow checker 也只会在这里为难你一次。

use rand::seq::IndexedRandom;
use std::collections::VecDeque;

pub const WIDTH: u16 = 20;
pub const HEIGHT: u16 = 12;

/// 方向。`enum` 替代 C 的 `#define UP 0` / Python 的字符串：
/// 值域在类型里穷举，`match` 时编译器强制全覆盖。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn opposite(self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

/// 坐标点。derive 自动实现比较，免去手写 `__eq__` / `memcmp`。
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

/// 游戏阶段。用 enum 而不是 `started: bool + over: bool` 两个标志：
/// (started=false, over=true) 这种非法组合在类型层面就不存在。
/// 对照 Python：要么靠 if 顺序，要么靠约定，编译器都不管。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Ready,
    Playing,
    Over,
}

pub struct Game {
    /// 蛇身：VecDeque 支持头尾两端 O(1) 操作，头在 front。
    /// 对照 Python list：appendleft/popleft 在 list 上是 O(n)，所以那边也用 deque。
    body: VecDeque<Pos>,
    dir: Dir,
    pub food: Pos,
    pub score: u32,
    pub phase: Phase,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Game {
            body: VecDeque::new(),
            dir: Dir::Right,
            food: Pos { x: 0, y: 0 },
            score: 0,
            phase: Phase::Ready,
        };
        g.reset();
        g
    }

    /// 开局/重开：回到初始状态。Ready 阶段蛇停着不动，
    /// 等玩家按第一个方向键才真正起步（见 turn）。
    pub fn reset(&mut self) {
        self.body.clear();
        self.body.push_front(Pos {
            x: WIDTH / 2,
            y: HEIGHT / 2,
        });
        self.dir = Dir::Right;
        self.score = 0;
        self.phase = Phase::Ready;
        self.place_food();
    }

    /// 就绪阶段收到方向键 → 用该方向开局；
    /// Playing 阶段 → 只允许非反向转向，非法输入直接丢弃。
    pub fn turn(&mut self, dir: Dir) {
        match self.phase {
            Phase::Ready => {
                self.dir = dir;
                self.phase = Phase::Playing;
            }
            Phase::Playing => {
                if dir != self.dir.opposite() {
                    self.dir = dir;
                }
            }
            Phase::Over => {}
        }
    }

    pub fn body(&self) -> &VecDeque<Pos> {
        &self.body
    }

    pub fn head(&self) -> Pos {
        // unwrap 之前已保证 body 非空（构造后放入、永不清空）——
        // 不变量由 impl 维护，可以安全 unwrap。
        *self.body.front().unwrap()
    }

    fn next_head(&self) -> Pos {
        let h = self.head();
        match self.dir {
            // saturating_add 防越出边界（撞墙判定在 step 里做）
            Dir::Up => Pos {
                x: h.x,
                y: h.y.saturating_sub(1),
            },
            Dir::Down => Pos { x: h.x, y: h.y + 1 },
            Dir::Left => Pos {
                x: h.x.saturating_sub(1),
                y: h.y,
            },
            Dir::Right => Pos { x: h.x + 1, y: h.y },
        }
    }

    /// 推进一步：移动 → 吃食或缩短 → 判死。
    /// 这是整个项目 borrow 最集中的一行：
    /// `self.body.contains(&next)` 读借用、`push_front`/`pop_back` 写借用，
    /// 顺序排好后编译器放行 —— 同时排好也天然正确。
    pub fn step(&mut self) {
        // 只在 Playing 阶段推进；Ready/Over 下 step 是空操作，
        // 调用方不必记住"哪些阶段能 step"——不变量自己守住。
        if self.phase != Phase::Playing {
            return;
        }

        let next = self.next_head();
        let hit_wall = next.x >= WIDTH || next.y >= HEIGHT;
        // 尾巴这一格下一刻就会让出来，所以撞上去不算咬到自己
        let will_grow = next == self.food;
        let mut body = self.body.iter().copied();
        if !will_grow {
            body.next_back(); // 去掉尾格再判咬身
        }
        // next == head 意味着被 saturating_* 钳在边界原地踏步 = 撞墙；
        // 不能靠咬身判：1 节蛇时尾格已被去掉，原地踏步会漏判成"卡住不死"。
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

    /// 在所有空格中均匀抽样。对照手写"重试直到空格"：
    /// 把候选先收集成 Vec，再让 rng.choose 一次性均匀取值——
    /// 无偏差、无重试循环，代码反而更短。
    fn place_food(&mut self) {
        let free: Vec<Pos> = (0..HEIGHT)
            .flat_map(|y| (0..WIDTH).map(move |x| Pos { x, y }))
            .filter(|p| !self.body.contains(p))
            .collect();
        self.food = *free
            .choose(&mut rand::rng())
            .expect("snake never fills the board in this tutorial");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_when_eating() {
        let mut g = Game::new();
        g.food = Pos {
            x: g.head().x + 1,
            y: g.head().y,
        }; // 食物放嘴边
        g.turn(Dir::Right); // 起步：Ready → Playing
        g.step();
        assert_eq!(g.score, 1);
        assert_eq!(g.body().len(), 2);
    }

    #[test]
    fn reverse_turn_is_ignored() {
        let mut g = Game::new();
        g.turn(Dir::Right); // Ready 阶段的第一个方向 = 起步方向
        g.turn(Dir::Left); // Playing 中向左是反向，无效
        assert_eq!(g.dir, Dir::Right);
    }

    #[test]
    fn dies_on_wall() {
        let mut g = Game::new();
        g.turn(Dir::Right); // Ready → Playing
        for _ in 0..WIDTH {
            g.step();
        }
        assert!(matches!(g.phase, Phase::Over)); // matches!：只判变体不取值
    }

    #[test]
    fn dies_when_pinned_to_edge() {
        // 回归：贴边界再反向（被 saturating_sub 钳住）必须判死，
        // 否则 1 节蛇会原地卡住永远不死。
        let mut g = Game::new();
        g.turn(Dir::Left); // 起步向左
        for _ in 0..WIDTH {
            g.step();
        }
        assert!(matches!(g.phase, Phase::Over));
    }
}
