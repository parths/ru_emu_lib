extern crate gl;
extern crate sdl2;

use sdl2:: { 
    Sdl, VideoSubsystem, 
    rect::{ Point }, 
    pixels::Color, 
    render::WindowCanvas, 
    event::Event, 
    keyboard::Keycode
};
//use std::thread;
use std::time::{ SystemTime };

use sdl_hello::emulators::{ chip8_emu, EmuTrait, ScreenResolution };

fn main() {
    let sdl = sdl2::init().unwrap();
    
    let video_subsystem = sdl.video().unwrap();

    try_sdl_canvas(&sdl, &video_subsystem);
    // print_sdl_debug_info(&sdl, &video_subsystem);
    // try_gl_loop(&sdl, &video_subsystem);
}

#[allow(unused_parens)]
fn try_sdl_canvas(sdl: &Sdl, video_subsystem: &VideoSubsystem) {
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
    // c8emu.load_data_file("/home/parosth-lenovo/partho/pLearn/rustuff/rusdl/ru_emu_lib/roms/IBMLogo.ch8");
    // c8emu.load_data_file("/home/parosth-lenovo/partho/pLearn/rustuff/rusdl/ru_emu_lib/roms/chip8-roms/demos/Stars [Sergey Naydenov, 2010].ch8");
    c8emu.load_data_file("/home/parosth-lenovo/partho/pLearn/rustuff/rusdl/ru_emu_lib/roms/Sierpinski [Sergey Naydenov, 2010].ch8");
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
        // for x in 1..10 {
        //     for y in  1..10 {
        //         let _ = canvas.draw_point(Point::new(x + pos_x as i32, y + pos_y as i32));
        //     }
        // }
        update_emulator(&mut c8emu);
        draw_emulator_screen(&mut canvas, &mut c8emu);
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