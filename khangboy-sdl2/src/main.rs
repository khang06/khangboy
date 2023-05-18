use imgui_glow_renderer::glow::{self, HasContext};
use khangboy_core::Gameboy;
use std::{fmt::Display, sync::mpsc, thread, time::Instant};

enum EmuThreadCommand {
    Quit,
}

// TODO: Syncing this stuff shouldn't happen if the windows are visible
#[derive(Clone)]
struct SharedData {
    registers: CPURegisters,

    tile_data: Box<[u8; 0x1800]>,
    tile_data_hash: u64,
}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            registers: Default::default(),
            tile_data: Box::new([0; 0x1800]),
            tile_data_hash: 0,
        }
    }
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
    const CLOCK_SPEED: u64 = 4194304 / 4;
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
            let tile_data = &gb.components.ppu.vram[..0x1800];
            let tile_data_hash = xxhash_rust::xxh3::xxh3_64(tile_data);
            let input = buf_input.input_buffer();
            input.registers.update(&gb.cpu);
            if input.tile_data_hash != tile_data_hash {
                input.tile_data.clone_from_slice(tile_data);
                input.tile_data_hash = tile_data_hash;
            }
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
        .window("khangboy-sdl2", 1366, 768)
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

    // Allocate tile data viewer texture
    let tile_tex = unsafe {
        let ctx = renderer.gl_context();
        let tex = ctx.create_texture()?;
        ctx.bind_texture(glow::TEXTURE_2D, Some(tex));
        ctx.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as _,
        );
        ctx.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as _,
        );
        ctx.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            16 * 8,
            24 * 8,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            None,
        );
        tex
    };

    // Spawn the emulation thread
    let (tx, rx) = mpsc::channel();
    let (buf_input, mut buf_output) = triple_buffer::triple_buffer(&Default::default());
    let rom_path = args[1].clone();
    thread::spawn(move || emu_thread(rom_path, buf_input, rx));

    // Run main event processing loop
    let mut event_pump = sdl.event_pump()?;
    let mut tile_data_temp = Box::new([0; (16 * 8) * (24 * 8) * 3]);
    let mut tile_data_hash = 0;
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

        if output.tile_data_hash != tile_data_hash {
            // TODO: The math here is horrible
            const COLORS: [u8; 4] = [0xFF, 0xC0, 0x40, 0x00];
            for y in 0..24 {
                for x in 0..16 {
                    for ty in 0..8 {
                        for tx in 0..8 {
                            let lo = (output.tile_data[(y * 128 + x * 8 + ty) * 2] >> (7 - tx))
                                as usize
                                & 1;
                            let hi = (output.tile_data[(y * 128 + x * 8 + ty) * 2 + 1] >> (7 - tx))
                                as usize
                                & 1;
                            let color = COLORS[(hi << 1) | lo];
                            tile_data_temp[((y * 8 + ty) * (16 * 8) + (x * 8 + tx)) * 3] = color;
                            tile_data_temp[((y * 8 + ty) * (16 * 8) + (x * 8 + tx)) * 3 + 1] =
                                color;
                            tile_data_temp[((y * 8 + ty) * (16 * 8) + (x * 8 + tx)) * 3 + 2] =
                                color;
                        }
                    }
                }
            }
            unsafe {
                renderer
                    .gl_context()
                    .bind_texture(glow::TEXTURE_2D, Some(tile_tex));
                renderer.gl_context().tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    16 * 8,
                    24 * 8,
                    glow::RGB,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(tile_data_temp.as_ref()),
                );
                //println!("{tile_data_temp:?}");
            }
            tile_data_hash = output.tile_data_hash;
        }

        ui.window("Registers")
            .size([90.0, 180.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text_wrapped(format!("{}", output.registers));
            });
        ui.window("PPU Tile Data")
            .size(
                [16.0 * 8.0 * 2.0 + 16.0, 24.0 * 8.0 * 2.0 + 36.0],
                imgui::Condition::FirstUseEver,
            )
            .build(|| {
                imgui::Image::new(
                    imgui::TextureId::new(tile_tex as usize),
                    [16.0 * 8.0 * 2.0, 24.0 * 8.0 * 2.0],
                )
                .build(ui);
            });

        let draw_data = imgui.render();
        unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
        renderer.render(draw_data).unwrap();

        window.gl_swap_window();

        let debug = unsafe { renderer.gl_context().get_debug_message_log(1024) };
        for x in debug {
            println!("{x:?}");
        }
    }

    // Signal the emulation thread to stop
    tx.send(EmuThreadCommand::Quit)?;

    Ok(())
}
