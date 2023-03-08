extern crate gl;
extern crate sdl2;

use sdl2:: { 
    Sdl, VideoSubsystem, 
    rect::{ Point }, 
    pixels::Color, 
    render::WindowCanvas, 
    event::Event, 
    keyboard::Keycode, 
};

//use std::thread;
use std::{ time::{ SystemTime }, env, process};

use ru_emu_lib::emulators::{ chip8_emu, EmuTrait, ScreenResolution, CpuInfo, RegisterSize, RegisterInfo };

mod p_bitmap_font;

fn main() {
    let sdl = sdl2::init().unwrap();
    
    let video_subsystem = sdl.video().unwrap();

    try_sdl_canvas(&sdl, &video_subsystem);
    // print_sdl_debug_info(&sdl, &video_subsystem);
    // try_gl_loop(&sdl, &video_subsystem);
}

fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

#[allow(unused_parens)]
fn try_sdl_canvas(sdl: &Sdl, video_subsystem: &VideoSubsystem) {

    let args: Vec<String> = env::args().collect();
    let is_debug_mode = args.contains(&String::from("debug"));
    let mut is_debug_paused = args.contains(&String::from("debug"));
    println!("[Debug] {}", is_debug_mode);

    let mut found_file_path = false;
    let mut file_path: String = String::from("");
    for arg in args {
        if arg == "--f" {
            found_file_path = true;
            continue;
        }
        if found_file_path {
            file_path = String::from(arg);
            break;
        }
    }
    if !found_file_path {
        println!("Please provide the file path using --f <file path>");
        process::exit(1);
    }

    let window = video_subsystem
        .window("RUST SDL OpenGL 00", 800, 600)
        .opengl()
        .build()
        .unwrap();
    println!("[Wnd]: {:?}", window.window_pixel_format());

    let mut canvas = window
        .into_canvas()
        .build()
        .unwrap();

    let mut c8emu = chip8_emu::Chip8Emu::new();
    c8emu.load_data_file(&file_path);
    let mut event_pump = sdl.event_pump().unwrap();

    let mov_x = 100.0;
    let mov_y = 100.0;
    let mut prev_delta_time = 0.0;
    let mut pos_x = 0.0;
    let mut pos_y = 0.0;

    'main: loop {
        let frame_start_time = SystemTime::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main, 
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Num1 => println!("1"), 
                        Keycode::Num2 => println!("2"), 
                        Keycode::Num3 => println!("3"), 
                        Keycode::Q => println!("Q"), 
                        Keycode::W => println!("W"), 
                        Keycode::E => println!("E"), 
                        Keycode::A => println!("A"), 
                        Keycode::S => println!("S"), 
                        Keycode::D => println!("D"), 
                        Keycode::Z => println!("Z"), 
                        Keycode::X => println!("X"), 
                        Keycode::C => println!("C"), 
                        Keycode::F10 => is_debug_paused = false, 
                        _ => {}
                    }
                }
                _ => {}, 
            }
        }

        pos_x += (prev_delta_time * mov_x);
        pos_y += (prev_delta_time * mov_y);
        if pos_x > 800.0 {
            pos_x = 0.0;
        }
        if pos_y > 600.0 {
            pos_y = 0.0;
        }

        canvas.set_draw_color(Color::RGB(64, 64, 64));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 0, 0));

        if !is_debug_paused {
            update_emulator(&mut c8emu);
        }
        is_debug_paused = is_debug_mode;
        draw_emulator_screen(&mut canvas, &mut c8emu);
        draw_cpu_info(&mut canvas, &mut c8emu);

        canvas.present();
        match frame_start_time.elapsed() {
            Ok(elapsed) => {
                prev_delta_time = elapsed.as_secs_f32();
            }, 
            Err(_) => {
            }, 
        }
    }
}

fn update_emulator(emu: &mut dyn EmuTrait)
{
    emu.tick();
}

fn draw_cpu_info(
    canvas: &mut WindowCanvas, 
    emu: &mut dyn CpuInfo
) {
    let reg_data = emu.get_data_registers();
    let mut i = 0;
    let x_offset = 4;
    let y_offset = 128;
    let fore_color = Color::RGB(255, 255, 0);
    let back_color = Color::RGB(2, 2, 2);
    let char_h_spacing = 2;
    let char_v_spacing = 2;
    for r_data in reg_data {
        let val_str = get_reg_value_hex(&r_data);
        let mut x_iter = 0;
        for c in val_str.chars() {
            p_bitmap_font::draw_letter(canvas, 
                x_offset + x_iter * (8 + char_h_spacing), 
                y_offset + i * (8 + char_v_spacing), 
                c as i32, &fore_color, &back_color);
            x_iter += 1;
        }
        i += 1;
    }
    let op_str = emu.get_current_instr();
    let mut x_iter = 0;
    for c in op_str.chars() {
        p_bitmap_font::draw_letter(canvas, 
            x_offset + x_iter * (8 + char_h_spacing), 
            y_offset + i * (8 + char_v_spacing), 
            c as i32, &fore_color, &back_color);
        x_iter += 1;
    }
    i += 1;
    let fore_color = Color::RGB(0, 255, 0);
    let back_color = Color::RGB(2, 2, 2);
    let op_str = emu.get_next_instr();
    let mut x_iter = 0;
    for c in op_str.chars() {
        p_bitmap_font::draw_letter(canvas, 
            x_offset + x_iter * (8 + char_h_spacing), 
            y_offset + i * (8 + char_v_spacing), 
            c as i32, &fore_color, &back_color);
        x_iter += 1;
    }
}

fn get_reg_value_hex(
    r_data: &RegisterInfo, 
) -> String {
    match r_data.reg_size_bits {
        RegisterSize::RegSize8 => format!("{:2x}", r_data.reg_value), 
        RegisterSize::RegSize16 => format!("{:4x}", r_data.reg_value), 
        RegisterSize::RegSize32 => format!("{:8x}", r_data.reg_value), 
        RegisterSize::RegSize64 => format!("{:16x}", r_data.reg_value), 
    }
}

fn draw_emulator_screen(
    canvas: &mut WindowCanvas, 
    emu: &mut dyn EmuTrait
) {
    let scale: u32 = 8; 

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let res = emu.get_screen_resolution();
    let w = res.width * scale;
    let h = res.height * scale;
    let screen_res = ScreenResolution { width: w, height: h};
    let mut draw_buf = vec![0; (w * h * 4) as usize];
    let pos_x = (800 - w) as i32 / 2;
    let pos_y = (600 - h) as i32 / 8;

    let draw_buf_result = emu.draw_to_buffer_rgba(&mut draw_buf, &screen_res);
    match draw_buf_result {
        Ok(_) => {
            for x in 0..w {
                for y in 0..h {
                    let byte_offset = (y * w * 4 + x * 4) as usize;
                    canvas.set_draw_color(Color::RGB(draw_buf[byte_offset], 
                        draw_buf[byte_offset + 1], draw_buf[byte_offset + 2]));

                    let _ = canvas.draw_point(Point::new(x as i32 + pos_x as i32, 
                        y as i32 + pos_y as i32));
                }
            }
        }, 
        Err(_) => {

        }
    }

    // let scr_rect = Rect::new((800 - w) as i32 / 2, (600 - h) as i32 / 8, w, h);
    // let _ = canvas.fill_rect(scr_rect);
}

fn _print_sdl_debug_info(_sdl: &Sdl, video_subsystem: &VideoSubsystem) {
    match video_subsystem.num_video_displays() {
        Ok(n_disp) => { 
            println!("[Displays]: {}", n_disp);
            for i in 0..n_disp {
                match video_subsystem.display_name(i) {
                    Ok(disp_name) => {
                        println!("\t[Display]: {}", disp_name);
                        let n_modes = video_subsystem.num_display_modes(i).unwrap();
                        for j in 0..n_modes {
                            match video_subsystem.display_mode(i, j) {
                                Ok(mode) => {
                                    println!("\t\t[DisplayMode]: {:?}", mode);
                                }, 
                                Err(err) => println!("[DisplayMode Error]: {}", err), 
                            }
                        }
                    }, 
                    Err(err) => println!("\t[Display Error]: {}", err), 
                }
            }
        }, 
        Err(_) => println!("WTF Displays!"), 
    };
}

fn _try_gl_loop(sdl: &Sdl, video_subsystem: &VideoSubsystem) {
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("RUST SDL OpenGL 00", 800, 600)
        .opengl()
        .build()
        .unwrap();
    println!("[Wnd]: {:?}", window.window_pixel_format());

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
    
    debug_assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    unsafe {
        gl::Viewport(0, 0, 800, 600);
        gl::ClearColor(0.0, 1.0, 0.0, 0.0);
    }
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main, 
                _ => {}, 
            }

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            window.gl_swap_window();
        }
    }
}