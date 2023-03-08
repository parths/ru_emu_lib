pub struct ScreenResolution {
    pub width: u32,
    pub height: u32,
}

pub enum RegisterSize
{
    RegSize8, 
    RegSize16, 
    RegSize32, 
    RegSize64, 
}

pub trait EmuTrait {
    fn load_data_file(self: &mut Self, file_name: &str);
    fn start(self: &mut Self);
    fn stop(self: &mut Self);
    fn pause(self: &mut Self);
    fn resume(self: &mut Self);
    fn get_screen_resolution(self: &Self) -> ScreenResolution;
    fn get_cpu_screen_resolution(self: &Self) -> ScreenResolution;

    fn draw_to_buffer_rgba(self: &Self, buf: &mut Vec<u8>, target_res: &ScreenResolution) 
        -> Result<bool, bool>;
    fn tick(self: &mut Self);
}

pub trait KeyboardDriver {
    fn on_key_press(self: &mut Self);
    fn on_key_release(self: &mut Self);
}

pub struct RegisterInfo {
    // pub reg_name: Box<str>, 
    pub reg_size_bits: RegisterSize, 
    pub reg_value: u64, 
}

pub trait CpuInfo {
    fn get_data_registers(self: &Self) -> Vec<RegisterInfo>;
    fn get_current_instr(self: &Self) -> String;
    fn get_next_instr(self: &Self) -> String;
}


pub mod chip8_emu;