use std::mem::transmute;

pub struct BitBuf {
    buf: [u8; 1400],
    pos: u16,       // The current bit position of the cursor.
    size: u16,      // Size in bits.
}

struct FourByte {
    b1: u8,
    b2: u8,
    b3: u8,
    b4: u8,
}

impl FourByte {
    pub fn trans_from_f32(value: f32) -> FourByte {
        unsafe { transmute::<f32, FourByte>(value) } 
    }

    pub fn trans_to_f32(self) -> f32 {
        unsafe { transmute::<FourByte, f32>(self) }
    }
}

struct EightByte {
    b1: u8,
    b2: u8,
    b3: u8,
    b4: u8,
    b5: u8,
    b6: u8,
    b7: u8,
    b8: u8,
}

impl EightByte {
    pub fn trans_from_f64(value: f64) -> EightByte {
        unsafe { transmute::<f64, EightByte>(value) } 
    }

    pub fn trans_to_f64(self) -> f64 {
        unsafe { transmute::<EightByte, f64>(self) }
    }
}

impl BitBuf {

    pub fn new() -> BitBuf {
        BitBuf {
            buf: [0u8; 1400],
            pos: 0,
            size: 1400,
        }
    }

    pub fn bit_size(&self) -> u16 {
        self.size
    }

    pub fn can_write_bits(&self, bit_size: u16) -> bool {
        (bit_size + self.pos) < self.size
    }

    pub fn can_read_bits(&self, bit_size: u16) -> bool {
        (bit_size + self.pos) < self.size
    }

    pub fn write_bool(&mut self, value: bool) {
        self.in_write_byte((if value {1} else {0}), 1);
    }

    pub fn read_bool(&mut self) -> bool {
        self.in_read_byte(1) == 1
    }

    pub fn write_i8(&mut self, value: i8) {
        self.write_i8_part(value, 8);
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read_i8_part(8)
    }

    pub fn write_i8_part(&mut self, value: i8, bits: u16) {
        self.in_write_byte(value as u8, bits);
    }

    pub fn read_i8_part(&mut self, bits: u16) -> i8 {
        self.in_read_byte(bits) as i8
    }

    pub fn write_u8(&mut self, value: u8) {
        self.write_u8_part(value, 8);
    }

    pub fn read_u8(&mut self) -> u8 {
        self.read_u8_part(8)
    }

    pub fn write_u8_part(&mut self, value: u8, bits: u16) {
        self.in_write_byte(value, bits);
    }

    pub fn read_u8_part(&mut self, bits: u16) -> u8 {
        self.in_read_byte(bits)
    }

    pub fn write_u16(&mut self, value: u16) {
        self.write_u16_part(value, 16);
    }

    pub fn read_u16(&mut self) -> u16 {
        self.read_u16_part(16)
    }

    pub fn write_u16_part(&mut self, value: u16, bits: u16) {
        if bits <= 8 {
            self.in_write_byte((value & 0xFF) as u8, bits);
        } else {
            self.in_write_byte((value & 0xFF) as u8, 8);
            self.in_write_byte((value >> 8) as u8, bits - 8);
        }
    }

    pub fn read_u16_part(&mut self, bits: u16) -> u16 {
        if bits <= 8 {
            self.in_read_byte(bits) as u16
        } else {
            (self.in_read_byte(8) as u16 | ((self.in_read_byte(bits - 8) as u16) << 8)) as u16
        }
    }

    pub fn write_i16(&mut self, value: i16) {
        self.write_i16_part(value, 16);
    }

    pub fn read_i16(&mut self) -> i16 {
        self.read_i16_part(16)
    }

    pub fn write_i16_part(&mut self, value: i16, bits: u16) {
        self.write_u16_part(value as u16, bits);
    }

    pub fn read_i16_part(&mut self, bits: u16) -> i16 {
        self.read_u16_part(bits) as i16
    }

    pub fn write_u32(&mut self, value: u32) {
        self.write_u32_part(value, 32);
    }

    pub fn read_u32(&mut self) -> u32 {
        self.read_u32_part(32)
    }

    pub fn write_u32_part(&mut self, value: u32, bits: u16) {
        let a = (value >> 0) as u8;
        let b = (value >> 8) as u8;
        let c = (value >> 16) as u8;
        let d = (value >> 24) as u8;

        match (bits + 7) / 8 {
            1 => {
                self.in_write_byte(a, bits);
            },
            2 => {
                self.in_write_byte(a, 8);
                self.in_write_byte(b, bits - 8);
            },
            3 => {
                self.in_write_byte(a, 8);
                self.in_write_byte(b, 8);
                self.in_write_byte(c, bits - 16);
            },
            4 => {
                self.in_write_byte(a, 8);
                self.in_write_byte(b, 8);
                self.in_write_byte(c, 8);
                self.in_write_byte(d, bits - 24);
            },
            _ => {
                //panic!("Must write between 1 and 32 bits.")
            }
        }
    }

    pub fn read_u32_part(&mut self, bits: u16) -> u32 {
        let mut a = 0i32;
        let mut b = 0i32;
        let mut c = 0i32;
        let mut d = 0i32;

        match (bits + 7) / 8 {
            1 => {
                a = self.in_read_byte(bits) as i32;
            },
            2 => {
                a = self.in_read_byte(8) as i32;
                b = self.in_read_byte(bits - 8) as i32;
            },
            3 => {
                a = self.in_read_byte(8) as i32;
                b = self.in_read_byte(8) as i32;
                c = self.in_read_byte(bits - 16) as i32;
            },
            4 => {
                a = self.in_read_byte(8) as i32;
                b = self.in_read_byte(8) as i32;
                c = self.in_read_byte(8) as i32;
                d = self.in_read_byte(bits - 24) as i32;
            },
            _ => {
                //panic!("Must read between 1 and 32 bits.")
            }
        }

        (a | (b << 8) | (c << 16) | (d << 24)) as u32
    }

    pub fn read_i32(&mut self) -> i32 {
        self.read_i32_part(32)
    }

    pub fn write_i32(&mut self, value: i32) {
        self.write_i32_part(value, 32);
    }

    pub fn write_i32_part(&mut self, value: i32, bits: u16) {
        self.write_u32_part(value as u32, bits);
    }

    pub fn read_i32_part(&mut self, bits: u16) -> i32 {
        self.read_u32_part(bits) as i32
    }

    pub fn write_u64(&mut self, value: u64) {
        self.write_u64_part(value, 64);
    }

    pub fn read_u64(&mut self) -> u64 {
        self.read_u64_part(64)
    }

    pub fn write_u64_part(&mut self, value: u64, bits: u16) {
        if bits <= 32 {
            self.write_u32_part((value & 0xFFFFFFFF) as u32, bits);
        } else {
            self.write_u32_part(value as u32, 32);
            self.write_u32_part((value >> 32) as u32, bits - 32);
        }
    }

    pub fn read_u64_part(&mut self, bits: u16) -> u64 {
        if bits <= 32 {
            self.read_u32_part(bits) as u64
        } else {
            let a = self.read_u32_part(32) as u64;
            let b = self.read_u32_part(bits - 32) as u64;
            a | (b << 32)
        }
    }

    pub fn write_i64(&mut self, value: i64) {
        self.write_u64_part(value as u64, 64);
    }

    pub fn read_i64(&mut self) -> i64 {
        self.read_u64_part(64) as i64
    }

    pub fn write_i64_part(&mut self, value: i64, bits: u16) {
        self.write_u64_part(value as u64, bits);
    }

    pub fn read_i64_part(&mut self, bits: u16) -> i64 {
        self.read_u64_part(bits) as i64
    }

    pub fn write_f32(&mut self, value: f32) {
        let trans = FourByte::trans_from_f32(value);
        self.in_write_byte(trans.b1, 8);
        self.in_write_byte(trans.b2, 8);
        self.in_write_byte(trans.b3, 8);
        self.in_write_byte(trans.b4, 8);
    }

    pub fn read_f32(&mut self) -> f32 {
        FourByte {
            b1: self.in_read_byte(8),
            b2: self.in_read_byte(8),
            b3: self.in_read_byte(8),
            b4: self.in_read_byte(8),
        }.trans_to_f32()
    }

    pub fn write_f64(&mut self, value: f64) {
        let trans = EightByte::trans_from_f64(value);
        self.in_write_byte(trans.b1, 8);
        self.in_write_byte(trans.b2, 8);
        self.in_write_byte(trans.b3, 8);
        self.in_write_byte(trans.b4, 8);
        self.in_write_byte(trans.b5, 8);
        self.in_write_byte(trans.b6, 8);
        self.in_write_byte(trans.b7, 8);
        self.in_write_byte(trans.b8, 8);
    }

    pub fn read_f64(&mut self) -> f64 {
        EightByte {
            b1: self.in_read_byte(8),
            b2: self.in_read_byte(8),
            b3: self.in_read_byte(8),
            b4: self.in_read_byte(8),
            b5: self.in_read_byte(8),
            b6: self.in_read_byte(8),
            b7: self.in_read_byte(8),
            b8: self.in_read_byte(8),
        }.trans_to_f64()
    }

    fn in_write_byte(&mut self, mut value: u8, bits: u16) {
        //if bits == 0 { panic!("Cannot write 0 bits."); }
        value = value & (0xFF >> (8 - bits));

        let p = (self.pos >> 3) as usize;
        let bits_used = self.pos & 0x7;
        let bits_free = 8 - bits_used;
        let bits_left = bits_free - bits;

        if bits_left >= 0 {
            let mask = (0xFF >> bits_free) | (0xFF << (8 - bits_left));
            self.buf[p] = (self.buf[p] & mask) | (value << bits_used);
        } else {
            self.buf[p] = (self.buf[p] & (0xFF >> bits_free)) | (value << bits_used);
            self.buf[p + 1] = (self.buf[p + 1] & (0xFF << (bits - bits_free))) | (value >> bits_free);
        }

        self.pos += bits;
    }

    fn in_read_byte(&mut self, bits: u16) -> u8 {
        let value: u8;
        let p = (self.pos >> 3) as usize;
        let bits_used = self.pos % 8;

        if bits_used == 0 && bits == 8 {
            value = self.buf[p];
        } else {
            let first = self.buf[p] >> bits_used;
            let remainder = bits - (8 - bits_used);
            if remainder < 1 {
                value = first & (0xFF >> (8 - bits));
            } else {
                let second = self.buf[p + 1] & (0xFF >> (8 - remainder));
                value = first | (second << (bits - remainder));
            }
        }

        self.pos += bits;
        value
    }

}

#[test]
fn bool_test() {
    let mut buf = BitBuf::new();
    let testval = true;
    buf.write_bool(testval);
    buf.pos = 0;
    assert!(buf.read_bool() == testval);
}

#[test]
fn u8_test() {
    let mut buf = BitBuf::new();
    let testval = 211;
    buf.write_u8(testval);
    buf.pos = 0;
    assert!(buf.read_u8() == testval);
}

#[test]
fn u8_part_test() {
    let mut buf = BitBuf::new();
    let testval = 15;
    buf.write_u8_part(testval, 4);
    buf.pos = 0;
    assert!(buf.read_u8_part(4) == testval);
}

#[test]
fn i8_part_test() {
    let mut buf = BitBuf::new();
    let testval = -6;
    buf.write_i8_part(testval, 4);
    buf.pos = 0;
    assert!(buf.read_i8_part(4) == testval);
}

#[test]
fn i8_test() {
    let mut buf = BitBuf::new();
    let testval = -109;
    buf.write_u8(testval);
    buf.pos = 0;
    assert!(buf.read_u8() == testval);
}

#[test]
fn u16_test() {
    let mut buf = BitBuf::new();
    let testval = 34507;
    buf.write_u16(testval);
    buf.pos = 0;
    assert!(buf.read_u16() == testval);
}

#[test]
fn u16_part_test() {
    let mut buf = BitBuf::new();
    let testval = 903;
    buf.write_u16_part(testval, 12);
    buf.pos = 0;
    assert!(buf.read_u16_part(12) == testval);
}

#[test]
fn i16_test() {
    let mut buf = BitBuf::new();
    let testval = -11066;
    buf.write_i16(testval);
    buf.pos = 0;
    assert!(buf.read_i16() == testval);
}

#[test]
fn i16_part_test() {
    let mut buf = BitBuf::new();
    let testval = -208;
    buf.write_i16_part(testval, 13);
    buf.pos = 0;
    let readval = buf.read_i16_part(13);
    println!("i16 read val: {} ", readval);
    assert!(readval == testval);
}

#[test]
fn u32_test() {
    let mut buf = BitBuf::new();
    let testval = 193772;
    buf.write_u32(testval);
    buf.pos = 0;
    assert!(buf.read_u32() == testval);
}

#[test]
fn i32_part_test() {
    let mut buf = BitBuf::new();
    let testval = -98;
    buf.write_i32_part(testval, 12);
    buf.pos = 0;
    assert!(buf.read_i32_part(12) == testval);
}

#[test]
fn f32_test() {
    let mut buf = BitBuf::new();
    let testval = 3.0393124f32;
    buf.write_f32(testval);
    buf.pos = 0;
    assert!(buf.read_f32() == testval);
}

#[test]
fn f64_test() {
    let mut buf = BitBuf::new();
    let testval = 3.0395831239485302f64;
    buf.write_f64(testval);
    buf.pos = 0;
    assert!(buf.read_f64() == testval);
}