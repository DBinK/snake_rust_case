//! 第 04 章：完整贪吃蛇。
//! 结构：main（循环）→ game（纯逻辑）→ terminal（I/O）。
//! 三态驱动：Ready 等第一个方向键 → Playing 循环 → Over 结算屏可重开。

mod game;
mod terminal;

use std::io;
use std::time::Duration;

use game::{Game, Phase};
use terminal::{Input, RawMode, clear, draw, read_input};

/// 起步节拍 200ms，每吃一块食物 -10ms，下限 60ms。
/// `saturating_sub` + `max`：减分提速但有下限，写进表达式本身。
const START_TICK_MS: u64 = 200;
const MIN_TICK_MS: u64 = 60;
const TICK_STEP_PER_POINT_MS: u64 = 10;

fn tick_for(score: u32) -> Duration {
    let ms = START_TICK_MS
        .saturating_sub(score as u64 * TICK_STEP_PER_POINT_MS)
        .max(MIN_TICK_MS);
    Duration::from_millis(ms)
}

fn main() -> io::Result<()> {
    // `let _guard`：接住守卫活到 main 结束；下划线前缀表明"故意不读它"。
    // 离开作用域时 Drop 自动恢复终端 —— 哪怕前面哪一步 `?` 提前返回。
    let _guard = RawMode::enter()?;
    clear()?;

    let mut g = Game::new();
    loop {
        draw(&g)?;
        match g.phase {
            // Ready：只等按键，无超时 —— 蛇停着，第一个方向键起步
            Phase::Ready => match read_input(Duration::from_secs(3600))? {
                Some(Input::Quit) => break,
                Some(Input::Move(d)) => g.turn(d), // 起步即转向
                _ => {}
            },
            // Playing：节拍随分数收紧，step 只在 Playing 生效（护栏在 Game 里）
            Phase::Playing => {
                match read_input(tick_for(g.score))? {
                    Some(Input::Quit) => break,
                    Some(Input::Move(d)) => g.turn(d),
                    _ => {}
                }
                g.step();
            }
            // Over：结算屏，r/回车重开，q 退出
            Phase::Over => match read_input(Duration::from_secs(3600))? {
                Some(Input::Quit) => break,
                Some(Input::Retry) => g.reset(),
                _ => {}
            },
        }
    }
    Ok(())
}
