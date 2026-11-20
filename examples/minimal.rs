//! 第 03 章：最小可跑版本 —— 一个能用方向键移动的 "@" 字符。
//! 目标：只学会三件事：进入/退出 raw mode、读键盘事件、把字符画到终端上。
//!
//! 对照 C：raw mode 相当于自己调 termios 关 ICANON/ECHO；
//! 对照 Python：相当于 curses.noecho() + getch()，但这里是显式的
//! 资源守卫（`terminal::enable_raw_mode()` 不是 RAII，需要我们手动恢复，
//! 所以结尾必须配对 disable —— 这也是第 04 章改用 guard 结构的原因）。

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    terminal::{self, ClearType},
};

fn main() -> io::Result<()> {
    // `?`：出错就把错误传回调用者，类似 Python 的异常向上抛，
    // 但它是编译期强制处理的 Result，不是运行时可能遗漏的 errno。
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut stdout = io::stdout();
    let (mut x, mut y) = (5u16, 5u16); // u16：坐标类型，负坐标在类型层面就不存在

    loop {
        // queue! 累积转义序列，flush 一次写出，避免逐字节系统调用
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(x, y)
        )?;
        stdout.write_all(b"@")?;
        stdout.flush()?;

        // `let Event::Key(key) = ... else`：2024 edition 的 let-else 语法，
        // “不是按键事件就跳过”，替代嵌套的 if let。
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
            // match 必须穷尽所有分支 —— 编译器替你检查“忘了处理的键”
            KeyCode::Up => y = y.saturating_sub(1), // saturating_sub：减到 0 为止，防下溢
            KeyCode::Down => y += 1,
            KeyCode::Left => x = x.saturating_sub(1),
            KeyCode::Right => x += 1,
            KeyCode::Char('q') | KeyCode::Esc => break, // `|` 模式或
            _ => {}                                     // `_` 兜底 = C 的 default
        }
    }

    // 恢复终端，否则你的 shell 会“坏掉”
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()
}
