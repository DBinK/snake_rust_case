//! 第 04 章：完整实现 —— 应用编排层与状态机主循环。
//!
//! 负责协调终端 I/O 适配层（terminal.rs）与纯领域逻辑层（game.rs）。
//! 采用状态机驱动的三态循环：
//! - Ready：长等待首个方向键起步。
//! - Playing：按动态节拍推进世界，接收转向按键。
//! - Over：结算展示，等待重开或退出。

mod game;
mod terminal;

use std::io;
use std::time::Duration;

use game::{Game, Phase};
use terminal::{Input, RawMode, clear, draw, read_input};

/// 初始起步节拍（200ms）。
const START_TICK_MS: u64 = 200;
/// 游戏最快步进节拍下限（60ms）。
const MIN_TICK_MS: u64 = 60;
/// 每得 1 分缩短的节拍步长（10ms）。
const TICK_STEP_PER_POINT_MS: u64 = 10;

/// 根据当前得分动态计算步进节拍周期。
/// 利用 saturating_sub 与 max 保证节拍在达到下限后不再缩短。
fn tick_for(score: u32) -> Duration {
    let ms = START_TICK_MS
        .saturating_sub(score as u64 * TICK_STEP_PER_POINT_MS)
        .max(MIN_TICK_MS);
    Duration::from_millis(ms)
}

fn main() -> io::Result<()> {
    // 实例化 RAII 守卫：当 _guard 离开当前作用域时，自动调用 Drop 恢复终端配置
    let _guard = RawMode::enter()?;
    clear()?;

    let mut g = Game::new();
    loop {
        draw(&g)?;
        match g.phase {
            // Ready 阶段：蛇身静止，挂起长超时等待玩家按下首个起步方向键
            Phase::Ready => match read_input(Duration::from_secs(3600))? {
                Some(Input::Quit) => break,
                Some(Input::Move(d)) => g.turn(d),
                _ => {}
            },
            // Playing 阶段：按照动态节拍超时轮询按键，并在每个周期推进一步
            Phase::Playing => {
                match read_input(tick_for(g.score))? {
                    Some(Input::Quit) => break,
                    Some(Input::Move(d)) => g.turn(d),
                    _ => {}
                }
                g.step();
            }
            // Over 阶段：进入游戏结算界面，支持重开或退出
            Phase::Over => match read_input(Duration::from_secs(3600))? {
                Some(Input::Quit) => break,
                Some(Input::Retry) => g.reset(),
                _ => {}
            },
        }
    }
    Ok(())
}
