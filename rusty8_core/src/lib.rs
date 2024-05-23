use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const START_ADDRESS: u16 = 0x0200;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Rusty {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_regs: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay: u8,
    sound: u8,
}

impl Rusty {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_regs: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay: 0,
            sound: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay = 0;
        self.sound = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, index: usize, pressed: bool) {
        self.keys[index] = pressed;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();

        self.ram[start..end].copy_from_slice(data);
    }
 
    //Fetch, decode, execute
    pub fn tick(&mut self) {
        //Fetch
        let op_code = self.fetch();
        //Decode and Execute
        self.execute(op_code);
    }

    //Fetch
    pub fn fetch(&mut self) -> u16 {
        let high_nibble = self.ram[self.pc as usize] as u16;
        let lower_nibble = self.ram[self.pc as usize] as u16;
        let op_code = (high_nibble << 8) | lower_nibble;

        self.pc += 2;
        op_code
    }

    //Decode and Execute
    fn execute(&mut self, op_code: u16) {
        let first_byte  = (op_code & 0xF000) >> 12;
        let second_byte = (op_code & 0x0F00) >> 8;
        let third_byte  = (op_code & 0x00F0) >> 4;
        let fourth_byte = op_code & 0x000F;

        match(first_byte, second_byte, third_byte, fourth_byte) {
            (0x0, 0x0, 0x0, 0x0) => return,
            (0x0, 0xE, 0x0, 0x0) => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.ret_sub(),
            (0x1,  _ ,  _ ,  _ ) => self.jp_addr(op_code & 0x0FFF),
            (0x2,  _ ,  _ ,  _ ) => self.call_addr(op_code & 0x0FFF),
            (0x3,  _ ,  _ ,  _ ) => self.skip_eq(second_byte as usize, (op_code & 0x00FF) as u8),
            (0x4,  _ ,  _ ,  _ ) => self.skip_neq(second_byte as usize, (op_code & 0x00FF) as u8),
            (0x5,  _ ,  _ , 0x0) => self.skip_reg_eq(second_byte as usize, third_byte as usize),
            (0x6,  _ ,  _ ,  _ ) => self.set_reg(second_byte as usize, (op_code & 0x00FF) as u8),
            (0x7,  _ ,  _ ,  _ ) => self.add_to_reg(second_byte as usize, (op_code & 0x00FF) as u8),
            (0x8,  _ ,  _ , 0x0) => self.ld_reg_to_reg(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0x1) => self.or_reg(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0x2) => self.and_reg(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0x3) => self.xor_reg(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0x4) => self.add_w_carry(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0x5) => self.sub_w_borrow(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0x6) => self.sr(second_byte as usize),
            (0x8,  _ ,  _ , 0x7) => self.sub_w_borrow_rev(second_byte as usize, third_byte as usize),
            (0x8,  _ ,  _ , 0xE) => self.sl(second_byte as usize),
            (0x9,  _ ,  _ , 0x0) => self.skip_reg_neq(second_byte as usize, third_byte as usize),
            (0xA,  _ ,  _ ,  _ ) => self.ld_i(op_code & 0x0FFF),
            (0xB,  _ ,  _ ,  _ ) => self.jp(op_code & 0x0FFF),
            (0xC,  _ ,  _ ,  _ ) => self.rand(second_byte as usize, (op_code & 0x00FF) as u8), 
            (0xD,  _ ,  _ ,  _ ) => self.draw_sprite(second_byte as usize, third_byte as usize, fourth_byte),
            (0xE,  _ , 0x9, 0xE) => self.skip_if_press(second_byte as usize),
            (0xE,  _ , 0xA, 0x1) => self.skip_not_press(second_byte as usize),
            (0xF,  _ , 0x0, 0x7) => self.store_delay(second_byte as usize),
            (0xF,  _ , 0x0, 0xA) => self.wait_key(second_byte as usize),
            (0xF,  _ , 0x1, 0x5) => self.set_delay(second_byte as usize),
            (0xF,  _ , 0x1, 0x8) => self.set_sound(second_byte as usize),
            (0xF,  _ , 0x1, 0xE) => self.add_i_reg(second_byte as usize),
            (0xF,  _ , 0x2, 0x9) => self.set_i_to_x(second_byte as usize),
            (0xF,  _ , 0x3, 0x3) => self.bcd(second_byte as usize),
            (0xF,  _ , 0x5, 0x5) => self.store_vreg(second_byte as usize),
            (0xF,  _ , 0x5, 0x6) => self.load_vreg(second_byte as usize),
            ( _ ,  _ ,  _ ,  _ ) => unimplemented!("Unimplemented opcode: {}", op_code)
        }
    }

    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn ret_sub(&mut self) {
        let ret_address = self.pop();
        self.pc = ret_address;
    }

    fn jp_addr(&mut self, val: u16) {
        self.pc = val;
    }

    fn call_addr(&mut self, val: u16) {
        self.push(self.pc);
        self.pc = val;
    }

    fn skip_eq(&mut self, reg: usize, val: u8) {
        let reg_val = self.v_regs[reg];
        if reg_val == val {
            self.pc += 2;
        }
    }

    fn skip_neq(&mut self, reg: usize, val: u8) {
        let reg_val = self.v_regs[reg];
        if reg_val != val {
            self.pc += 2;
        }
    }

    fn skip_reg_eq(&mut self, reg_1: usize, reg_2: usize) {
        if self.v_regs[reg_1] == self.v_regs[reg_2] {
            self.pc += 2;
        }
    }

    fn set_reg(&mut self, reg: usize, val: u8) {
        self.v_regs[reg] = val;
    }

    fn add_to_reg(&mut self, reg: usize, val: u8) {
        self.v_regs[reg] = self.v_regs[reg].wrapping_add(val);
    }

    fn ld_reg_to_reg(&mut self, reg_1: usize, reg_2: usize) {
        self.v_regs[reg_1] = self.v_regs[reg_2];
    }

    fn or_reg(&mut self, reg_1: usize, reg_2: usize) {
        self.v_regs[reg_1] |= self.v_regs[reg_2];
    }

    fn and_reg(&mut self, reg_1: usize, reg_2: usize) {
        self.v_regs[reg_1] &= self.v_regs[reg_2];
    }

    fn xor_reg(&mut self, reg_1: usize, reg_2: usize) {
        self.v_regs[reg_1] ^= self.v_regs[reg_2];
    }

    fn add_w_carry(&mut self, reg_1: usize, reg_2: usize) {
        let (new_reg_1, carry) = self.v_regs[reg_1].overflowing_add(self.v_regs[reg_2]);
        let vf = if carry { 1 } else { 0 };

        self.v_regs[reg_1] = new_reg_1;
        self.v_regs[0xF] = vf;
    }

    fn sub_w_borrow(&mut self, reg_1: usize, reg_2: usize) { 
        let (new_reg_1, borrow) = self.v_regs[reg_1].overflowing_sub(self.v_regs[reg_2]);
        let vf = if borrow { 0 } else { 1 };

        self.v_regs[reg_1] = new_reg_1;
        self.v_regs[0xF] = vf;
    }

    fn sr(&mut self, reg: usize) {
        let lsb = self.v_regs[reg] & 0x1;

        self.v_regs[reg] >>= 1;
        self.v_regs[0xF] = lsb;
    }

    fn sub_w_borrow_rev(&mut self, reg_1: usize, reg_2: usize) {
        let (new_reg_1, borrow) = self.v_regs[reg_2].overflowing_sub(self.v_regs[reg_1]);
        let vf = if borrow { 0 } else { 1 };

        self.v_regs[reg_1] = new_reg_1;
        self.v_regs[0xF] = vf;
    }

    fn sl(&mut self, reg: usize) {
        let msb = (self.v_regs[reg] & 0x80) >> 7;

        self.v_regs[reg] <<= 1;
        self.v_regs[0xF] = msb;
    }

    fn skip_reg_neq(&mut self, reg_1: usize, reg_2: usize) {
        if self.v_regs[reg_1] != self.v_regs[reg_2] {
            self.pc += 2;
        }
    }

    fn ld_i(&mut self, val: u16) {
        self.i_reg = val;
    }

    fn jp(&mut self, val: u16) {
        self.pc = (self.v_regs[0x0] as u16) + val;
    }

    fn rand(&mut self, reg: usize, val: u8) {
        let random_u8: u8 = random();

        self.v_regs[reg] = random_u8 & val;
    }

    fn draw_sprite(&mut self, reg_1: usize, reg_2: usize, rows: u16) {
        let x: u16 = self.v_regs[reg_1] as u16;
        let y = self.v_regs[reg_2] as u16;

        let mut pixel_flipped = false;

        for y_line in 0..rows {
            let address = self.i_reg + y_line as u16;
            let pixels  = self.ram[address as usize];

            for x_line in 0..8 {
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    let pixel_x = (x + x_line as u16) as usize % SCREEN_WIDTH;
                    let pixel_y = (y + y_line as u16) as usize % SCREEN_HEIGHT;

                    let index = pixel_x + SCREEN_WIDTH * pixel_y;

                    pixel_flipped |= self.screen[index];
                    self.screen[index] ^= true;
                }
            }
        }

        if pixel_flipped {
            self.v_regs[0xF] = 1;
        } else {
            self.v_regs[0xF] = 0;
        }
    }

    fn skip_if_press(&mut self, reg: usize) {
        let key = self.keys[self.v_regs[reg] as usize];

        if key {
            self.pc += 2;
        }
    }

    fn skip_not_press(&mut self, reg: usize) { 
        let key = self.keys[self.v_regs[reg] as usize];

        if !key {
            self.pc += 2;
        }
    }

    fn store_delay(&mut self, reg: usize) {
        self.v_regs[reg] = self.delay;
    }

    fn wait_key(&mut self, reg: usize) {
        let mut pressed = false;
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_regs[reg] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.pc -= 2;
        }
    }

    fn set_delay(&mut self, reg: usize) {
        self.delay = self.v_regs[reg];
    }

    fn set_sound(&mut self, reg: usize) {
        self.sound = self.v_regs[reg];
    }

    fn add_i_reg(&mut self, reg: usize) {
        self.i_reg = self.i_reg.wrapping_add(self.v_regs[reg] as u16);
    }

    fn set_i_to_x(&mut self, reg: usize) {
        let reg_value = self.v_regs[reg] as u16;

        self.i_reg = reg_value * 5;
    }

    fn bcd(&mut self, reg: usize) {
        let reg_value = self.v_regs[reg] as f32;

        let hundreds = (reg_value / 100.0).floor() as u8;
        let tens = ((reg_value / 10.0) % 10.0).floor() as u8;
        let ones = (reg_value % 10.0) as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }   

    fn store_vreg(&mut self, reg: usize) {
        for index in 0..reg {
            self.ram[self.i_reg as usize + index] = self.v_regs[index];
        }
    }

    fn load_vreg(&mut self, reg: usize) {
        for index in 0..reg {
            self.v_regs[index] = self.ram[self.i_reg as usize + index];
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }

        if self.sound > 0 {
            if self.sound == 1 {
                //Beep
            }

            self.sound -= 1;
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
