//! 终端适配层：Raw Mode 模式生命周期、屏幕双缓冲绘制与非阻塞事件解析。
//!
//! 将所有终端特定依赖隔离在此模块，确保领域核心 game.rs 的纯粹与可移植性。

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

/// RAII 终端资源守卫。
/// 构造时进入 Raw Mode 并切换到交替屏幕，离开作用域时通过 Drop 自动恢复。
pub struct RawMode;

impl RawMode {
    /// 激活 Raw Mode 并切换至交替屏幕缓冲区。
    pub fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(RawMode)
    }
}

impl Drop for RawMode {
    /// 析构函数：保证程序无论是正常退出还是因错误提前返回，终端均能确定性恢复。
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// 抽象按键事件枚举。
/// 将具体终端按键映射为领域关注的高层意图。
pub enum Input {
    Move(Dir),
    Quit,
    Retry,
}

/// 非阻塞输入事件轮询。
/// - 在给定超时内无按键事件时返回 Ok(None)。
/// - 解析到有效控制按键时返回对应的 Input 变体。
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

/// 单帧渲染：聚合控制转义序列与网格字符，最后一次性刷盘。
pub fn draw(g: &Game) -> io::Result<()> {
    let mut out = io::stdout();
    queue!(out, cursor::MoveTo(0, 0))?;

    // 绘制顶部分数栏与操作提示
    queue!(
        out,
        Print(format!(
            "  Score: {}   (h/j/k/l or arrows, q to quit)\r\n",
            g.score
        ))
    )?;

    // 逐行绘制棋盘网格
    for y in 0..HEIGHT {
        queue!(out, Print("  "))?;
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
        // Raw Mode 下换行不附带回车，必须显式输出 \r\n
        queue!(out, Print("\r\n"))?;
    }

    // 根据当前阶段绘制底部状态横幅
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

/// 清除终端全部内容。
pub fn clear() -> io::Result<()> {
    queue!(io::stdout(), terminal::Clear(ClearType::All))?;
    io::stdout().flush()
}
