use chip8_core::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEventKind, poll, read};
use crossterm::style::{PrintStyledContent, Stylize};
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{cursor, execute, queue};
use std::env;
use std::fs::File;
use std::io::{Read, Write, stdout};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), std::io::Error> {
    enable_raw_mode()?;
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "examples/Sierpinski [Sergey Naydenov, 2010].ch8"
    };

    let mut file = File::open(filename).expect("ROM file not found");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    let mut emu = Emu::new();
    emu.load_rom(&buffer);

    println!("ROM loaded: {} bytes", buffer.len());
    println!("Press Esc to exit.");

    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Hide,
        EnterAlternateScreen,
        Clear(ClearType::All)
    )?;

    loop {
        static FPS: u64 = 60;
        let frame_duration = Duration::from_micros(1_000_000 / FPS);
        let instructions_per_frame = 500 / FPS;
        let frame_start = Instant::now();

        emu.reset_keys();
        if poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('1') => emu.keys[0x1] = true,
                        KeyCode::Char('2') => emu.keys[0x2] = true,
                        KeyCode::Char('3') => emu.keys[0x3] = true,
                        KeyCode::Char('4') => emu.keys[0xC] = true,

                        KeyCode::Char('q') => emu.keys[0x4] = true,
                        KeyCode::Char('w') => emu.keys[0x5] = true,
                        KeyCode::Char('e') => emu.keys[0x6] = true,
                        KeyCode::Char('r') => emu.keys[0xD] = true,

                        KeyCode::Char('a') => emu.keys[0x7] = true,
                        KeyCode::Char('s') => emu.keys[0x8] = true,
                        KeyCode::Char('d') => emu.keys[0x9] = true,
                        KeyCode::Char('f') => emu.keys[0xE] = true,

                        KeyCode::Char('z') => emu.keys[0xA] = true,
                        KeyCode::Char('x') => emu.keys[0x0] = true,
                        KeyCode::Char('c') => emu.keys[0xB] = true,
                        KeyCode::Char('v') => emu.keys[0xF] = true,

                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        for _ in 0..instructions_per_frame {
            if let Err(()) = emu.fetch() {
                break;
            }
        }

        emu.update_timers();

        display(&emu)?;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            sleep(frame_duration - elapsed);
        }
    }
    execute!(stdout, cursor::Show, LeaveAlternateScreen,)?;
    disable_raw_mode()?;
    Ok(())
}

fn display(emu: &Emu) -> Result<(), std::io::Error> {
    const DOT: &str = "██";
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let index = y * SCREEN_WIDTH + x;
            if emu.screen[index] {
                queue!(
                    stdout(),
                    MoveTo(x as u16 * 2, y as u16),
                    PrintStyledContent(DOT.white())
                )?;
            } else {
                queue!(
                    stdout(),
                    MoveTo(x as u16 * 2, y as u16),
                    PrintStyledContent(DOT.black())
                )?;
            }
        }
    }
    queue!(
        stdout(),
        MoveTo(0, SCREEN_HEIGHT as u16 + 1),
        PrintStyledContent("Press Esc to exit.".white())
    )?;
    stdout().flush()?;
    Ok(())
}
