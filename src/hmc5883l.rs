use embedded_hal::blocking::i2c::{Write, WriteRead};

// 0x3c for write, 0x3d for read
const IIC_ADDR: u8 = 0x1e;

#[repr(u8)]
pub enum RegistersWriteable {
	ConA = 0x00,
	ConB = 0x01,
	Mode = 0x02,
}

pub enum DatumRegisters {
	X,
	Y,
	Z,
}

impl DatumRegisters {
	fn get8(&self) -> (u8, u8) {
		match self {
			DatumRegisters::X => (0x03, 0x04),
			DatumRegisters::Y => (0x05, 0x06),
			DatumRegisters::Z => (0x07, 0x08),
		}
	}
}

#[repr(u8)]
pub enum RegistersNonWr {
	Status = 0x09,
	IdA = 0x10,
	IdB = 0x11,
	IdC = 0x12,
}

#[repr(u8)]
pub enum OpMode {
	Continuous = 0x00,
	Single = 0x01,
	Idle = 0x02,
}

pub struct Datum {
	pub x: u16,
	pub y: u16,
	pub z: u16,
}

pub struct Hmc5883l<I2C> {
	i2c: I2C,
}

impl<I2C, E> Hmc5883l<I2C>
where
	I2C: Write<Error = E> + WriteRead<Error = E>,
{
	pub fn new(i2c: I2C) -> Self {
		Hmc5883l { i2c }
	}

	// This device supports standard and fast modes, 100kHz
	// and 400kHz, respectively, but does not support the high speed mode (Hs).
	// [...]
	// Set this pin to enable High Speed I2C, 3400kHz.
	//
	// i'm including this function just because i'm confused
	pub fn write_hs_mode(&mut self, is_hs: bool) {
		let reg = self.read_reg(RegistersWriteable::Mode as u8);
		let val = reg & ((is_hs as u8) << 7);
		let _ = self.i2c.write(IIC_ADDR, &[RegistersWriteable::Mode as u8, val]);
	}

	pub fn set_mode(&mut self, mode: OpMode) {
		// this is unnecessary if the hs bit does nothing
		let reg = self.read_reg(RegistersWriteable::Mode as u8);
		let val = reg & mode as u8;
		let _ = self.i2c.write(IIC_ADDR, &[RegistersWriteable::Mode as u8, val]);
	}

	// yikes
	// i could make it so that i can only pass an enum value which has valid register addresses
	// but then accessing 16-bit addresses would be less convenient
	fn read_reg(&mut self, reg: u8) -> u8 {
		let mut buf: [u8; 1] = [0; 1];
		let _ = self.i2c.write_read(IIC_ADDR, &[reg], &mut buf);
		buf[0]
	}

	pub fn get_angle(&mut self, axis: DatumRegisters) -> u16 {
		let (hi, lo) = axis.get8();
		let hi = self.read_reg(hi);
		let lo = self.read_reg(lo);
		(hi << 8) as u16 & lo as u16
	}

	pub fn get_angles(&mut self) -> (u16, u16, u16) {
		(
			self.get_angle(DatumRegisters::X),
			self.get_angle(DatumRegisters::Y),
			self.get_angle(DatumRegisters::Z),
		)
	}

	// pub fn get_mode(&mut self) -> OpMode {
	// 	match self.get_mode_reg() {
	// 		0b00000000 => OpMode::Continuous,
	// 		0b00000001 => OpMode::Single,
	// 		0b00000011 => OpMode::Idle,
	// 		_ => panic!(),
	// 	}
	// }
}
