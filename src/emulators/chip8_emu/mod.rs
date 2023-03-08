use std::fs;
use std::fs::File;
use std::io::Read;
use super::{ EmuTrait, ScreenResolution, CpuInfo, RegisterInfo, RegisterSize };
use rand::Rng;

const PROGRAM_START: usize = 0x200;

pub struct Chip8Emu {
    memory: Vec<u8>, 
    reg: Vec<u8>, 
    // stack: Vec<u16>, 
    display_buffer: Vec<u8>, 
    program_counter: u16, 
    index_register: u16, 
    // stack_pointer: u8, 
    delay_timer: u8, 
    sound_timer: u8, 

    is_running: bool, 
    keys: Vec<bool>, 
    wait_for_key: bool, 
    font_sprite_offset: u16, 

    curr_opcode: u16, 
}

impl CpuInfo for Chip8Emu {
    fn get_data_registers(self: &Self) -> Vec<RegisterInfo> {
        let mut c_info = Vec::<RegisterInfo>::new();
        for i in 0..16 {
            c_info.push(RegisterInfo {
                reg_size_bits: RegisterSize::RegSize8, 
                reg_value: self.reg[i] as u64, 
            });
        }
        c_info.push(RegisterInfo {
            reg_size_bits: RegisterSize::RegSize16, 
            reg_value: self.program_counter as u64, 
        });
        c_info.push(RegisterInfo {
            reg_size_bits: RegisterSize::RegSize16, 
            reg_value: self.index_register as u64, 
        });
        c_info
    }

    fn get_current_instr(self: &Self) -> String {
        self.translate_opcode(self.curr_opcode)
    }
    fn get_next_instr(self: &Self) -> String {
        let mut opcode: u16 = (self.memory[self.program_counter as usize] & 0xff) as u16;
        opcode = opcode << 8;
        opcode = opcode | ((self.memory[(self.program_counter + 1) as usize] & 0xff) as u16);
        self.translate_opcode(opcode)
    }
}

impl EmuTrait for Chip8Emu {
    fn start(self: &mut Self) {
        self.program_counter = 0x200;
        self.is_running = true;
    }

    fn stop(self: &mut Self) {
        self.is_running = false;
    }

    fn pause(self: &mut Self) {
        self.is_running = false;
    }

    fn resume(self: &mut Self) {
        self.is_running = true;
    }

    fn load_data_file(self: &mut Self, file_name: &str) {
        println!("[Loading] {}...", file_name);
        let mut f = File::open(file_name).expect("Could not open file!");
        let metadata = fs::metadata(file_name).expect("Could not read metadata... ");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("Could not read file!");

        // println!("[Got Data]:\n{:x?}", buffer);
        for i in 0..buffer.len() {
            self.memory[i + PROGRAM_START] = buffer[i];
        }

        self.is_running = false;
        self.program_counter = 0x200;
    }

    fn get_screen_resolution(self: &Self) -> ScreenResolution {
        ScreenResolution {
            width: 64, 
            height: 32, 
        }
    }

    fn get_cpu_screen_resolution(self: &Self) -> ScreenResolution {
        ScreenResolution {
            // 2 chars for register name
            // + 4 chars for register value
            // + 1 space char
            // * 8 pixels per char
            // * 1 pixel space per char (rounded to 8 pixels total)
            width: 64, 
            // 1 row for each register
            // + 1 row for pc
            // + 1 row for index reg (IR)
            // + 1 row for delay timer
            // + 1 row for audio timer
            height: 184, 
        }
    }

    fn draw_to_buffer_rgba(self: &Self, buf: &mut Vec<u8>, target_res: &ScreenResolution) 
        -> Result<bool, bool> {

        let screen_res = self.get_screen_resolution();
        let pixel_width = target_res.width / screen_res.width;
        let pixel_height = target_res.height / screen_res.height;

        let mut display_buffer_x;
        let mut display_buffer_y;

        for x in 0..target_res.width {
            display_buffer_x = x / pixel_width;
            for y in 0..target_res.height {
                display_buffer_y = y / pixel_height;
                let arr_offset = y * 4 * target_res.width + (x * 4);
                let val = self.display_buffer[(display_buffer_y * screen_res.width + display_buffer_x) as usize];
                buf[(arr_offset) as usize] = val;
                buf[(arr_offset + 1) as usize] = val;
                buf[(arr_offset + 2) as usize] = val;
            }
        }

        Ok(true)
    }

    fn tick(self: &mut Self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        if !self.wait_for_key {
            // let pc = self.program_counter;
            self.curr_opcode = self.fetch_opcode();
            // println!("0x{:4x} - {}", pc, self.translate_opcode(opcode));
            self.exec_opcode(self.curr_opcode);
            if self.wait_for_key {
                self.program_counter -= 2;
            }
        }
    }
}

impl Chip8Emu {
    pub fn new() -> Chip8Emu {
        Chip8Emu {
            memory: vec![0; 0x1000],        // 4096 bytes
            reg: vec![0; 0x10],             // 16 bytes
            // stack: vec![0; 0x30],           // 48 short
            display_buffer: vec![0; 0x800],  // 64 * 32 bytes 
            program_counter: 0, 
            index_register: 0, 
            // stack_pointer: 0, 
            delay_timer: 0, 
            sound_timer: 0, 

            is_running: false, 
            keys: vec![false; 0xff], 
            wait_for_key: false, 
            font_sprite_offset: 0, 

            curr_opcode: 0, 
        }
    }

    fn fetch_opcode(self: &mut Self) -> u16 {
        let mut opcode: u16 = (self.memory[self.program_counter as usize] & 0xff) as u16;
        opcode = opcode << 8;
        opcode = opcode | ((self.memory[(self.program_counter + 1) as usize] & 0xff) as u16);
        self.program_counter += 2;
        opcode
    }

    fn translate_opcode(self: &Self, opcode: u16) -> String {
        let op = ((opcode & 0xf000) >> 12, 
        (opcode & 0x0f00) >> 8, 
        (opcode & 0x00f0) >> 4, 
        (opcode & 0x000f));

        match op {
            (0x0, 0x0, 0xe, 0x0) => format!("CLRSCR"), 
            (0x0, 0x0, 0xe, 0xe) => format!("RET"), 
            (0x0, n1, n2, n3) => format!("CALLM 0x{:4x}", ((n1 << 8) | (n2 << 4) | n3)), 
            (0x1, n1, n2, n3) => format!("JMP 0x{:4x}", ((n1 << 8) | (n2 << 4) | n3)), 
            (0x2, n1, n2, n3) => format!("CALL 0x{:4x}", ((n1 << 8) | (n2 << 4) | n3)), 
            (0x3, x, n1, n2) => format!("JEQ V{} 0x{:4x}", x, ((n1 << 4) | n2)), 
            (0x4, x, n1, n2) => format!("JNE V{} 0x{:4x}", x, ((n1 << 4) | n2)), 
            (0x5, x, y, 0) => format!("JEQ V{} V{}", x, y), 
            (0x6, x, n1, n2) => format!("SET V{} 0x{:4x}", x, ((n1 << 4) | n2)), 
            (0x7, x, n1, n2) => format!("ADD V{} 0x{:4x}", x, ((n1 << 4) | n2)), 
            (0x8, x, y, 0x0) => format!("MOV V{} V{}", x, y), 
            (0x8, x, y, 0x1) => format!("OR V{} V{}", x, y), 
            (0x8, x, y, 0x2) => format!("AND V{} V{}", x, y), 
            (0x8, x, y, 0x3) => format!("XOR V{} V{}", x, y), 
            (0x8, x, y, 0x4) => format!("ADD V{} V{}", x, y), 
            (0x8, x, y, 0x5) => format!("SUB V{} V{}", x, y), 
            (0x8, x, _y, 0x6) => format!("SHR V{}", x), 
            (0x8, x, y, 0x7) => format!("SUBD V{} V{}", y, x), 
            (0x8, x, y, 0xe) => format!("SHL V{} V{}", x, y), 
            (0x9, x, y, 0x0) => format!("JNE V{} V{}", x, y), 
            (0xa, n1, n2, n3) => format!("SETI 0x{:4x}", ((n1 << 8) | (n2 << 4) | n3)), 
            (0xb, n1, n2, n3) => format!("JMP 0x{:4x}", ((n1 << 8) | (n2 << 4) | n3)), 
            (0xc, x, n1, n2) => format!("RND V{} 0x{:4x}", x, ((n1 << 4) | n2)), 
            (0xd, x, y, n) => format!("DRAW V{} V{} 0x{:4x}", x, y, n), 
            (0xe, x, 0x9, 0xe) => format!("JKEY V{}", x), 
            (0xe, x, 0xa, 0x1) => format!("JNKEY V{}", x), 
            (0xf, x, 0x0, 0x7) => format!("GETDELAY V{}", x), 
            (0xf, x, 0x0, 0xa) => format!("GETKEY V{}", x), 
            (0xf, x, 0x1, 0x5) => format!("SETDELAY V{}", x), 
            (0xf, x, 0x1, 0x8) => format!("SETSOUND V{}", x), 
            (0xf, x, 0x1, 0xe) => format!("ADDI V{}", x), 
            (0xf, x, 0x2, 0x9) => format!("SETI V{}", x), 
            (0xf, x, 0x3, 0x3) => format!("BCD V{}", x), 
            (0xf, x, 0x5, 0x5) => format!("STRMEM V{}", x), 
            (0xf, x, 0x6, 0x5) => format!("LDMEM V{}", x), 
            (_, _, _, _) => format!("INVALID: 0x{:4x}", opcode)
        }
    }

    fn exec_opcode(self: &mut Self, opcode: u16) {
        if opcode == 0x00e0 {
            self.op_00e0_cls();
        } else if (opcode & 0xf000) == 0x1000 {
            self.op_1nnn_jmp(opcode);
        } else if (opcode & 0xf000) == 0x3000 {
            self.op_3xnn_je(opcode);
        } else if (opcode & 0xf000) == 0x4000 {
            self.op_4xnn_jne(opcode);
        } else if (opcode & 0xf000) == 0x5000 {
            self.op_5xy0_je(opcode);
        } else if (opcode & 0xf000) == 0x6000 {
            self.op_6xnn_set_reg(opcode);
        } else if (opcode & 0xf000) == 0x7000 {
            self.op_7xnn_add_reg(opcode);
        } else if (opcode & 0xf00f) == 0x8000 {
            self.op_8xy0_assign_reg(opcode);
        } else if (opcode & 0xf00f) == 0x8001 {
            self.op_8xy1_or(opcode);
        } else if (opcode & 0xf00f) == 0x8002 {
            self.op_8xy2_and(opcode);
        } else if (opcode & 0xf00f) == 0x8003 {
            self.op_8xy3_xor(opcode);
        } else if (opcode & 0xf00f) == 0x8004 {
            self.op_8xy4_add(opcode);
        } else if (opcode & 0xf00f) == 0x8005 {
            self.op_8xy5_sub(opcode);
        } else if (opcode & 0xf00f) == 0x8006 {
            self.op_8xy6_rshift(opcode);
        } else if (opcode & 0xf00f) == 0x8007 {
            self.op_8xy7_sub(opcode);
        } else if (opcode & 0xf000) == 0x9000 {
            self.op_9xy0_jne(opcode);
        } else if (opcode & 0xf00f) == 0x800e {
            self.op_8xye_lshift(opcode);
        } else if (opcode & 0xf000) == 0xa000 {
            self.op_annn_set_index(opcode);
        } else if (opcode & 0xf000) == 0xb000 {
            self.op_bnnn_jmp(opcode);
        } else if (opcode & 0xf000) == 0xc000 {
            self.op_cxnn_rnd(opcode);
        } else if (opcode & 0xf000) == 0xd000 {
            self.op_dxyn_display(opcode);
        } else if (opcode & 0xf0ff) == 0xe09e {
            self.op_ex9e_jmp_key_on(opcode);
        } else if (opcode & 0xf0ff) == 0xe0a1 {
            self.op_exa1_jmp_key_off(opcode);
        } else if (opcode & 0xf0ff) == 0xf007 {
            self.op_fx07_get_delay_timer(opcode);
        } else if (opcode & 0xf0ff) == 0xf00a {
            self.op_fx0a_get_key(opcode);
        } else if (opcode & 0xf0ff) == 0xf015 {
            self.op_fx15_set_delay_timer(opcode);
        } else if (opcode & 0xf0ff) == 0xf018 {
            self.op_fx18_set_sound_timer(opcode);
        } else if (opcode & 0xf0ff) == 0xf01e {
            self.op_fx1e_add_to_index(opcode);
        } else if (opcode & 0xf0ff) == 0xf029 {
            self.op_fx29_set_sprite_to_index(opcode);
        } else if (opcode & 0xf0ff) == 0xf033 {
            self.op_fx33_bcd(opcode);
        } else if (opcode & 0xf0ff) == 0xf055 {
            self.op_fx55_reg_dump(opcode);
        } else if (opcode & 0xf0ff) == 0xf065 {
            self.op_fx65_reg_load(opcode);
        } else { 
            println!("[Not implemented]: {:4x}", opcode);
        }
    }

    /**
     * Implement op-codes
     */ 
    fn op_00e0_cls(self: &mut Self) {
        for i in 0..0x800 {
            self.display_buffer[i] = 0;
        }
    }

    fn op_1nnn_jmp(self: &mut Self, opcode: u16) {
        let nnn = (opcode & 0x0fff) as u16;
        self.program_counter = nnn;
    }

    fn op_3xnn_je(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let nn = (opcode & 0xff) as u8;
        if self.reg[x as usize] == nn {
            self.program_counter += 2;
        }
    }

    fn op_4xnn_jne(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let nn = (opcode & 0xff) as u8;
        if self.reg[x as usize] != nn {
            self.program_counter += 2;
        }
    }

    fn op_5xy0_je(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        if self.reg[x as usize] == self.reg[y as usize] {
            self.program_counter += 2;
        }
    }

    fn op_6xnn_set_reg(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let nn = (opcode & 0xff) as u8;
        self.reg[x as usize] = nn;
    }

    fn op_7xnn_add_reg(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let nn = (opcode & 0xff) as u8;
        self.reg[x as usize] += nn;
    }

    fn op_8xy0_assign_reg(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        self.reg[x as usize] = self.reg[y as usize];
    }

    fn op_8xy1_or(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        self.reg[x as usize] = self.reg[x as usize] | self.reg[y as usize];
    }

    fn op_8xy2_and(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        self.reg[x as usize] = self.reg[x as usize] & self.reg[y as usize];
    }

    fn op_8xy3_xor(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        self.reg[x as usize] = self.reg[x as usize] ^ self.reg[y as usize];
    }

    fn op_8xy4_add(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        let z: u16 = self.reg[x as usize] as u16 
            + self.reg[y as usize] as u16;
        if (z & 0xff00) > 0 {
            self.reg[15] = 1
        } else {
            self.reg[15] = 0;
        }
        self.reg[x as usize] = (z & 0xff) as u8;
    }

    fn op_8xy5_sub(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        let z: i16 = self.reg[x as usize] as i16 
            - self.reg[y as usize] as i16;
        if self.reg[x as usize] < self.reg[y as usize] {
            self.reg[15] = 0
        } else {
            self.reg[15] = 1;
        }
        self.reg[x as usize] = (z & 0xff) as u8;
    }

    fn op_8xy6_rshift(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.reg[15] = self.reg[x as usize] & 0x01;
        self.reg[x as usize] = self.reg[x as usize] >> 1;
    }

    fn op_8xy7_sub(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        let z: u16 = self.reg[y as usize] as u16 
            - self.reg[x as usize] as u16;
        if self.reg[x as usize] >= self.reg[y as usize] {
            self.reg[15] = 1
        } else {
            self.reg[15] = 0;
        }
        self.reg[x as usize] = (z & 0xff) as u8;
    }

    fn op_8xye_lshift(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.reg[15] = (self.reg[x as usize] & 0x80) >> 7;
        self.reg[x as usize] = self.reg[x as usize] << 1;
    }

    fn op_9xy0_jne(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let y = ((opcode & 0xf0) >> 4) as u8;
        if self.reg[x as usize] != self.reg[y as usize] {
            self.program_counter += 2;
        }
    }

    fn op_annn_set_index(self: &mut Self, opcode: u16) {
        self.index_register = opcode & 0xfff;
    }

    fn op_dxyn_display(self: &mut Self, opcode: u16) {
        let vx: u16 = ((opcode & 0x0f00) >> 8).try_into().unwrap();
        let vy: u16 = ((opcode & 0xf0) >> 4) as u16;
        let n: u8 = (opcode & 0x0f) as u8;
        let x_pos = self.reg[vx as usize] as u16;
        let y_pos = self.reg[vy as usize] as u16;

        for y in 0..n {
            let row_val = self.memory[(self.index_register + y as u16) as usize];
            let draw_pos: usize = (((y_pos + y as u16) * 64) + x_pos).into();
            for x in (0..8).rev() {
                let bit_val = (row_val >> x) & 0x01;
                if bit_val == 1 {
                    self.reg[15] = 1;
                    self.display_buffer[(draw_pos + (7 - x)) as usize] = 
                        !self.display_buffer[(draw_pos + (7 - x)) as usize];
                }
            }
        }
    }

    fn op_bnnn_jmp(self: &mut Self, opcode: u16) {
        let nnn: u16 = opcode & 0x0fff;
        self.program_counter = self.reg[0] as u16 + nnn;
    }

    fn op_cxnn_rnd(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let nn = (opcode & 0xff) as u8;
        self.reg[x as usize] = rand::thread_rng().gen_range(0..=nn);
    }

    fn op_ex9e_jmp_key_on(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        if self.keys[self.reg[x as usize] as usize] {
            self.program_counter += 2;
        }
    }

    fn op_exa1_jmp_key_off(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        if !self.keys[self.reg[x as usize] as usize] {
            self.program_counter += 2;
        }
    }

    fn op_fx07_get_delay_timer(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.reg[x as usize] = self.delay_timer;
    }

    fn op_fx0a_get_key(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let mut key:i16 = -1;
        for k in 0..self.keys.len() {
            if self.keys[k] {
                key = k as i16;
                break;
            }
        }
        if key >= 0 {
            self.reg[x as usize] = (key & 0x0f) as u8;
        } else {
            self.wait_for_key = true;
        }
    }

    fn op_fx15_set_delay_timer(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.delay_timer = self.reg[x as usize];
    }

    fn op_fx18_set_sound_timer(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.sound_timer = self.reg[x as usize];
    }

    fn op_fx1e_add_to_index(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.index_register += self.reg[x as usize] as u16;
    }

    fn op_fx29_set_sprite_to_index(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        self.index_register = self.font_sprite_offset 
            + (self.reg[x as usize] * 5) as u16;
    }

    fn op_fx33_bcd(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let mut x_val = self.reg[x as usize];
        let mut i = self.index_register;
        while x_val > 0 {
            self.memory[i as usize] = x_val % 10;
            x_val = x_val / 10;
            i += 1;
        }
        self.index_register = self.font_sprite_offset 
            + (self.reg[x as usize] * 5) as u16;
    }

    fn op_fx55_reg_dump(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let mut mem_pos = self.index_register;
        for i in 0..=x {
            self.memory[mem_pos as usize] = self.reg[i as usize];
            mem_pos += 1;
        }
    }

    fn op_fx65_reg_load(self: &mut Self, opcode: u16) {
        let x = (opcode & 0x0f00) >> 8;
        let mut mem_pos = self.index_register;
        for i in 0..=x {
            self.reg[i as usize] = self.memory[mem_pos as usize];
            mem_pos += 1;
        }
    }
}