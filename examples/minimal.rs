//! 第 03 章：最小可跑版本 —— 终端交互单字符移动原型。
//!
//! 演示终端游戏最核心的三项底层能力：
//! 1. 切换至 Raw Mode 并建立交替屏幕缓冲区。
//! 2. 非阻塞或单次阻塞读取键盘事件流。
//! 3. 定位光标并在指定坐标绘制字符。
//!
//! 对照说明：
//! - C 语言：需要手动调用 termios API 清除 ICANON 与 ECHO 标志位。
//! - Python：通常依赖 curses 库的 noecho() 与 cbreak()。
//! - Rust：直接调用 crossterm 提供的声明式 API，但注意手动退出模式存在泄漏风险，
//!   完整版在第 04 章将进一步升级为 RAII 守卫模式。

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    terminal::{self, ClearType},
};

fn main() -> io::Result<()> {
    // 进入 Raw Mode 并切换到交替屏幕缓冲区，隐藏光标以保证画面纯净
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut stdout = io::stdout();
    // 使用 u16 无符号整数存储坐标，从类型层面规避负数坐标的发生
    let (mut x, mut y) = (5u16, 5u16);

    loop {
        // queue! 宏批量写入清屏与光标跳转指令，flush 一次性刷盘，减少系统调用
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(x, y)
        )?;
        stdout.write_all(b"@")?;
        stdout.flush()?;

        // 使用 Rust 2024 let-else 语法过滤非按键事件，早退扁平化分支
        let Event::Key(key) = event::read()? else {
            continue;
        };

        // match 穷尽检查：编译器强制覆盖 KeyCode 的分支处理
        match key.code {
            KeyCode::Up => y = y.saturating_sub(1), // saturating_sub 达到 0 时自动饱和，防止无符号下溢
            KeyCode::Down => y += 1,
            KeyCode::Left => x = x.saturating_sub(1),
            KeyCode::Right => x += 1,
            KeyCode::Char('q') | KeyCode::Esc => break, // 多模式匹配组合
            _ => {}                                     // 兜底分支忽略其他按键
        }
    }

    // 显式恢复终端状态，避免污染外部 Shell 交互环境
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()
}
