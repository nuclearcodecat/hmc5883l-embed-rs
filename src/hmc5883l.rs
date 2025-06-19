use stm32f1xx_hal::i2c::BlockingI2c;
use embedded_hal::blocking::i2c::Write;

const IIC_BASE_ADDR: u8 = 0x1e;

const IIC_WRITE_ADDR: u8 = 0x3c;
const IIC_READ_ADDR: u8 = 0x3d;

const MODE_REGISTER: u8 = 0x02;

pub enum OpModes {
	Continuous,
	Single,
	Idle,
}

pub struct Datum {
	pub x: u16,
	pub y: u16,
	pub z: u16,
}

pub struct Hmc5883l<I2C> {
	i2c: I2C,
}

impl<I2C> Hmc5883l<I2C> 
where 
	I2C: Write,
{
	pub fn new(i2c: I2C) -> Self {
		Hmc5883l { i2c }
	}

	pub fn set_mode(&mut self, mode: OpModes) {
		match mode {
			OpModes::Continuous => {
				self.i2c
					.write(IIC_WRITE_ADDR, &[MODE_REGISTER, 0b00000000]);
			},
			OpModes::Single => {
				self.i2c
					.write(IIC_WRITE_ADDR, &[MODE_REGISTER, 0b00000001]);
			},
			OpModes::Idle => {
				self.i2c
					.write(IIC_WRITE_ADDR, &[MODE_REGISTER, 0b00000011]);
			},
		}
	}
}
