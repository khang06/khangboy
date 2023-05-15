use gb::Gameboy;
use rom::rom_from_bytes;

mod apu;
mod components;
mod cpu;
mod gb;
mod ppu;
mod rom;
mod serial;
mod timer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Some(path) = args.get(1) {
        let mut gb = Gameboy::new(rom_from_bytes(&std::fs::read(path).unwrap()).unwrap());
        gb.run();
    } else {
        println!("Usage: khangboy [rom]");
    }
}
