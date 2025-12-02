use std::env;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

use chip8_core::Emu;

fn main() {
    let mut emu = Emu::new();

    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "../examples/IBM Logo.ch8"
    };

    let mut file = File::open(filename).expect("ROM file not found");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    emu.load_rom(&buffer);

    println!("ROM loaded: {} bytes", buffer.len());
    println!("Press Ctrl+C to exit.");

    print!("\x1B[2J");

    loop {
        emu.fetch();
        emu.display();
        thread::sleep(Duration::from_millis(16));
    }
}
