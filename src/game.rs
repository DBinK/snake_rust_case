//! 纯领域逻辑层：贪吃蛇核心状态与规则演变。
//!
//! 本模块完全不包含任何 I/O、系统调用或终端交互逻辑。
//! 这种纯函数式/状态机设计让领域规则可以进行毫秒级的确定性单元测试。

use rand::seq::IndexedRandom;
use std::collections::VecDeque;

/// 棋盘网格宽度（单元格数）。
pub const WIDTH: u16 = 20;
/// 棋盘网格高度（单元格数）。
pub const HEIGHT: u16 = 12;

/// 行进方向枚举。
/// 通过代数数据类型穷举所有可能的行进方向，编译器保证模式匹配的完整性。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    /// 获取当前方向的反方向，用于 180 度掉头拦截。
    fn opposite(self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

/// 棋盘网格二维坐标点。
/// derive 特征自动提供基于值的相等性比较与复制能力。
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

/// 游戏生命周期阶段状态机。
/// 借助枚举彻底消除多布尔标志位组合带来的非法废态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Ready,
    Playing,
    Over,
}

/// 游戏核心实体状态。
pub struct Game {
    /// 蛇身节点双端队列，front 对应蛇头，back 对应蛇尾。
    body: VecDeque<Pos>,
    /// 当前行进方向。
    dir: Dir,
    /// 食物所在坐标。
    pub food: Pos,
    /// 当前累计得分。
    pub score: u32,
    /// 当前所处阶段。
    pub phase: Phase,
}

impl Game {
    /// 创建全新的游戏实例并初始化为 Ready 阶段。
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

    /// 重置游戏状态回到初始就绪态。
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

    /// 接收方向变更输入。
    /// - Ready 阶段：设定起步方向并直接转换至 Playing 阶段。
    /// - Playing 阶段：拦截 180 度反向调头，仅接受合法转向。
    /// - Over 阶段：忽略所有转向请求。
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

    /// 获取蛇身队列的不可变引用，供渲染模块读取。
    pub fn body(&self) -> &VecDeque<Pos> {
        &self.body
    }

    /// 获取蛇头坐标。
    /// 结构体不变式保证 body 在初始化后永不为空，可安全解包。
    pub fn head(&self) -> Pos {
        *self.body.front().unwrap()
    }

    /// 根据当前朝向计算蛇头下一个目标网格坐标。
    fn next_head(&self) -> Pos {
        let h = self.head();
        match self.dir {
            // saturating_sub 避免无符号整数减法下溢
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

    /// 游戏世界步进一步：移动蛇身、检测碰撞并处理吃食成长。
    ///
    /// 本方法展示了 Rust 借用检查器的核心设计：
    /// 先通过不可变迭代视图完成碰撞与吃食判定，再执行可变队列修改。
    pub fn step(&mut self) {
        // 守卫条件：仅在 Playing 阶段推进游戏物理步进
        if self.phase != Phase::Playing {
            return;
        }

        let next = self.next_head();
        let hit_wall = next.x >= WIDTH || next.y >= HEIGHT;
        let will_grow = next == self.food;

        // 使用轻量双端迭代器建立只读视图
        let mut body = self.body.iter().copied();
        if !will_grow {
            // 未吃到食物时尾节点即将移出，因此从判定视图末尾排除尾部
            body.next_back();
        }

        // next == self.head() 用于捕获单节蛇贴边移动时被 saturating_sub 钳制在原地的撞墙情况
        let hit_self = next == self.head() || body.any(|p| p == next);

        if hit_wall || hit_self {
            self.phase = Phase::Over;
            return;
        }

        // 判定通过后执行可变写入
        self.body.push_front(next);
        if will_grow {
            self.score += 1;
            self.place_food();
        } else {
            self.body.pop_back();
        }
    }

    /// 在棋盘未被占用的空格中均匀随机生成新食物。
    /// 采用全量空格收集后无偏采样，避免盲目重试产生概率偏斜与极端死循环。
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
        // 人工将食物放置在蛇头正右方
        g.food = Pos {
            x: g.head().x + 1,
            y: g.head().y,
        };
        g.turn(Dir::Right);
        g.step();
        assert_eq!(g.score, 1);
        assert_eq!(g.body().len(), 2);
    }

    #[test]
    fn reverse_turn_is_ignored() {
        let mut g = Game::new();
        g.turn(Dir::Right);
        // 向左为反向操作，在 Playing 阶段应当被忽略
        g.turn(Dir::Left);
        assert_eq!(g.dir, Dir::Right);
    }

    #[test]
    fn dies_on_wall() {
        let mut g = Game::new();
        g.turn(Dir::Right);
        for _ in 0..WIDTH {
            g.step();
        }
        assert!(matches!(g.phase, Phase::Over));
    }

    #[test]
    fn dies_when_pinned_to_edge() {
        // 回归测试：贴边并向外侧行进时，必须被判定为撞墙死亡，杜绝单节蛇卡住不死的缺陷
        let mut g = Game::new();
        g.turn(Dir::Left);
        for _ in 0..WIDTH {
            g.step();
        }
        assert!(matches!(g.phase, Phase::Over));
    }
}
