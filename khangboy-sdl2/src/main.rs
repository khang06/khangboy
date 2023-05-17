use imgui_glow_renderer::glow::{self, HasContext};
use khangboy_core::Gameboy;
use std::{
    fmt::Display,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

enum EmuThreadCommand {
    Quit,
}

#[derive(Clone, Default)]
struct SharedData {
    registers: CPURegisters,
}

#[derive(Clone, Default)]
struct CPURegisters {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,

    sp: u16,
    pc: u16,
}

impl CPURegisters {
    fn update(&mut self, cpu: &khangboy_core::cpu::CPU) {
        self.a = cpu.a;
        self.b = cpu.b;
        self.c = cpu.c;
        self.d = cpu.d;
        self.e = cpu.e;
        self.f = cpu.f;
        self.h = cpu.h;
        self.l = cpu.l;
        self.sp = cpu.sp;
        self.pc = cpu.pc;
    }
}

impl Display for CPURegisters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A:  0x{:02X}\nB:  0x{:02X}\nC:  0x{:02X}\nD:  0x{:02X}\nE:  0x{:02X}\nF:  0x{:02X}\nH:  0x{:02X}\nL:  0x{:02X}\nSP: 0x{:04X}\nPC: 0x{:04X}",
            self.a, self.b, self.c, self.d, self.e, self.f, self.h, self.l, self.sp, self.pc)
    }
}

fn emu_thread(
    rom_path: String,
    mut buf_input: triple_buffer::Input<SharedData>,
    rx: mpsc::Receiver<EmuThreadCommand>,
) {
    let rom = khangboy_core::rom::rom_from_bytes(&std::fs::read(rom_path).unwrap()).unwrap();
    let mut gb = Gameboy::new(rom);

    // ~2ms per timestep
    const CLOCK_SPEED: u64 = 4194304;
    const TARGET_CYCLES: u64 = CLOCK_SPEED / 512;

    let start = Instant::now();
    let mut cycles_executed = 0;
    loop {
        // Handle any messages from the main thread
        if let Ok(msg) = rx.try_recv() {
            match msg {
                EmuThreadCommand::Quit => break,
            }
        }

        // Run the emulator
        // TODO: This method of throttling kinda sucks
        let cycles_to_run = loop {
            let diff = (Instant::now().duration_since(start).as_secs_f64() * CLOCK_SPEED as f64)
                as u64
                - cycles_executed;
            if diff >= TARGET_CYCLES {
                break diff;
            }
        };
        cycles_executed += gb.run(cycles_to_run);

        // Update the shared data
        {
            let input = buf_input.input_buffer();
            input.registers.update(&gb.cpu);
        }
        buf_input.publish();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: khangboy-sdl2 [rom]");
        return Ok(());
    }

    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;

    // Make SDL2 create an OpenGL 3.3 core profile context
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    // Make a new window with OpenGL support
    let window = video_subsystem
        .window("Hello imgui-rs!", 1366, 768)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .build()?;

    // Make a new OpenGL context
    let gl_context = window.gl_create_context()?;
    window.gl_make_current(&gl_context)?;
    window.subsystem().gl_set_swap_interval(1)?;

    // Create Glow and ImGui contexts
    let gl = unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    };
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    // Create ImGui platform and renderer
    let mut platform = imgui_sdl2_support::SdlPlatform::init(&mut imgui);
    let mut renderer = imgui_glow_renderer::AutoRenderer::initialize(gl, &mut imgui)?;

    // Spawn the emulation thread
    let (tx, rx) = mpsc::channel();
    let (buf_input, mut buf_output) = triple_buffer::triple_buffer(&Default::default());
    let rom_path = args[1].clone();
    thread::spawn(move || emu_thread(rom_path, buf_input, rx));

    // Run main event processing loop
    let mut event_pump = sdl.event_pump()?;
    'main: loop {
        for event in event_pump.poll_iter() {
            platform.handle_event(&mut imgui, &event);

            if let sdl2::event::Event::Quit { .. } = event {
                break 'main;
            }
        }

        platform.prepare_frame(&mut imgui, &window, &event_pump);

        let ui = imgui.new_frame();
        ui.show_demo_window(&mut true);

        buf_output.update();
        let output = buf_output.output_buffer();
        ui.window("Registers")
            .size([90.0, 180.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text_wrapped(format!("{}", output.registers));
            });

        let draw_data = imgui.render();
        unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
        renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }

    // Signal the emulation thread to stop
    tx.send(EmuThreadCommand::Quit)?;

    Ok(())
}
