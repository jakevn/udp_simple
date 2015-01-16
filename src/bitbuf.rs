
pub struct BitBuf {
	buf: [u8; 1400],
	pos: u16,       // The current bit position of the cursor.
	size: u16, 	    // Size in bits.
}

impl BitBuf {

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
			(self.in_read_byte(8) | (self.in_read_byte(bits - 8) << 8)) as u16
		}
	}

	pub fn write_i16_part(&mut self, value: i16, bits: u16) {
		self.write_u16_part(value as u16, bits);
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