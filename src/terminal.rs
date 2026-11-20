//! 终端层：raw mode、渲染、输入，全部隔离在这个文件里。
//! game.rs 不知道终端的存在 —— 换 GUI 只改这里。

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    queue,
    style::Print,
    terminal::{self, ClearType},
};

use crate::game::{Dir, Game, HEIGHT, Phase, Pos, WIDTH};

/// RAII 守卫：构造时进入 raw mode，**离开作用域自动恢复**。
/// 这是 Rust 版 Python `with` / C++ 的 RAII：
/// 不管 main 是正常返回还是 ? 提前退出，Drop 都会跑。
pub struct RawMode;

impl RawMode {
    pub fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(RawMode)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// 输入事件：enum 让"可能的结果"成为类型的一部分。
/// 调用方 match 时必须处理每个变体 —— 想"忘了写退出键"都做不到。
/// Retry 只在结算屏有意义，主循环里 match 到它直接忽略。
pub enum Input {
    Move(Dir),
    Quit,
    Retry,
}

/// 非阻塞读输入：`poll` 等最多 `timeout`，没按键就返回 None。
/// Option<Input> 显式表达"可能没有"，替代 Python 返回 None 的约定 ——
/// 调用方不处理它就编译不过。
pub fn read_input(timeout: Duration) -> io::Result<Option<Input>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    let input = match event::read()? {
        Event::Key(k) => match k.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Input::Move(Dir::Up)),
            KeyCode::Down | KeyCode::Char('j') => Some(Input::Move(Dir::Down)),
            KeyCode::Left | KeyCode::Char('h') => Some(Input::Move(Dir::Left)),
            KeyCode::Right | KeyCode::Char('l') => Some(Input::Move(Dir::Right)),
            KeyCode::Char('q') | KeyCode::Esc => Some(Input::Quit),
            KeyCode::Char('r') | KeyCode::Enter => Some(Input::Retry),
            _ => None,
        },
        _ => None,
    };
    Ok(input)
}

/// 一帧渲染：queue! 攒齐转义序列，最后一次 flush。
pub fn draw(g: &Game) -> io::Result<()> {
    let mut out = io::stdout();
    queue!(out, cursor::MoveTo(0, 0))?;
    // 顶行：分数栏
    queue!(
        out,
        Print(format!(
            "  Score: {}   (h/j/k/l or arrows, q to quit)\r\n",
            g.score
        ))
    )?;
    // 棋盘逐行打印：墙 + 蛇身 + 食物
    for y in 0..HEIGHT {
        queue!(out, Print("  "))?; // 左边距
        for x in 0..WIDTH {
            let p = Pos { x, y };
            let cell = if g.body().contains(&p) {
                'O'
            } else if p == g.food {
                '*'
            } else {
                '.'
            };
            queue!(out, Print(cell))?;
        }
        queue!(out, Print("\r\n"))?; // raw mode 下 \n 不回车，必须 \r\n
    }
    match g.phase {
        Phase::Ready => queue!(
            out,
            Print("  Ready — press an arrow key to start, q to quit\r\n")
        )?,
        Phase::Over => queue!(
            out,
            Print(format!(
                "  Game Over! score={}   r: retry   q: quit\r\n",
                g.score
            ))
        )?,
        Phase::Playing => {}
    }
    out.flush()
}

/// 清屏（开局/退出前用）。
pub fn clear() -> io::Result<()> {
    queue!(io::stdout(), terminal::Clear(ClearType::All))?;
    io::stdout().flush()
}
