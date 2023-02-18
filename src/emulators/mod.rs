pub struct ScreenResolution {
    pub width: u32,
    pub height: u32,
}

pub trait EmuTrait {
    fn load_data_file(self: &mut Self, file_name: &str);
    fn start(self: &mut Self);
    fn stop(self: &mut Self);
    fn pause(self: &mut Self);
    fn resume(self: &mut Self);
    fn get_screen_resolution(self: &Self) -> ScreenResolution;

    fn draw_to_buffer_rgba(self: &Self, buf: &mut Vec<u8>, target_res: &ScreenResolution) 
        -> Result<bool, bool>;
    fn tick(self: &mut Self);
}

pub trait KeyboardDriver {
    fn on_keypress(self: &mut Self)
}

pub mod chip8_emu;