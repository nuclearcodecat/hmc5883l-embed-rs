// todo  do something about the panics
// todo  embedded_hal 1.0.0 support
#![no_std]

use embedded_hal::blocking::i2c::{Write, WriteRead, Read};

// 0x3c for write, 0x3d for read
const IIC_ADDR: u8 = 0x1e;

const AVG_MASK: u8 = 0b01100000;
const ODR_MASK: u8 = 0b00011100;
const MOD_MASK: u8 = 0b00000011;

#[repr(u8)]
enum Registers {
	ConA,
	ConB,
	Mode,
	HiX,
	LoX,
	HiZ,
	LoZ,
	HiY,
	LoY,
	Status,
	IdA,
	IdB,
	IdC,
}

pub enum Axes {
	X,
	Y,
	Z,
}

impl Axes {
	fn get_regs8(&self) -> (u8, u8) {
		match self {
			Axes::X => (Registers::HiX as u8, Registers::LoX as u8),
			Axes::Z => (Registers::HiZ as u8, Registers::LoZ as u8),
			Axes::Y => (Registers::HiY as u8, Registers::LoY as u8),
		}
	}
}

#[repr(u8)]
pub enum OperationModes {
	Continuous,
	Single,
	Idle,
}

#[repr(u8)]
pub enum AveragedSamples {
	One,
	Two,
	Four,
	Eight,
}

#[repr(u8)]
pub enum OutputRates {
	Hz0_75,
	Hz1_5,
	Hz3_0,
	Hz7_5,
	Hz15_0,
	Hz30_0,
	Hz75_0,
	// Reserved,
}

#[repr(u8)]
pub enum MeasurementModes {
	Normal,
	PositiveBias,
	NegativeBias,
	// Reserved,
}

#[repr(u8)]
pub enum Gain {
	_1370,
	_1090,
	_820,
	_660,
	_440,
	_390,
	_330,
	_220,
}

pub struct Hmc5883l<I2C> {
	i2c: I2C,
}

impl<I2C, E> Hmc5883l<I2C>
where
	I2C: Write<Error = E> + WriteRead<Error = E> + Read<Error = E>,
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
	pub fn write_hs_mode(&mut self, is_hs: bool) -> Result<(), E>  {
		let reg = self.read_reg(Registers::Mode as u8)?;
		let val = (reg & !(1 << 7)) | ((is_hs as u8) << 7);
		self.i2c.write(IIC_ADDR, &[Registers::Mode as u8, val])
	}

	pub fn set_operating_mode(&mut self, mode: OperationModes) -> Result<(), E> {
		// this is unnecessary if the hs bit does nothing
		let reg = self.read_reg(Registers::Mode as u8)?;
		let val = (reg & !0x03) | mode as u8;
		self.i2c.write(IIC_ADDR, &[Registers::Mode as u8, val])
	}

	// yikes
	// i could make it so that i can only pass an enum value which has valid register addresses
	// but then accessing 16-bit addresses would be less convenient
	fn read_reg(&mut self, reg: u8) -> Result<u8, E> {
		let mut buf: [u8; 1] = [0; 1];
		self.i2c.write_read(IIC_ADDR, &[reg], &mut buf)?;
		Ok(buf[0])
	}

	// seems like it would be a good idea to call this with interrupts maybe
	// i'm thinking - data can change between hi and lo read
	pub fn get_angle(&mut self, axis: Axes) -> Result<i16, E> {
		let (hi, lo) = axis.get_regs8();
		let hi = self.read_reg(hi)?;
		let lo = self.read_reg(lo)?;
		Ok(i16::from_be_bytes([hi, lo]))
	}

	pub fn get_angles(&mut self) -> Result<(i16, i16, i16), E> {
		let mut buf = [0u8; 6];
		self.i2c.write_read(IIC_ADDR, &[Registers::HiX as u8], &mut buf)?;
		Ok(( 
			// xzy order in registers
			i16::from_be_bytes([buf[0], buf[1]]),
			i16::from_be_bytes([buf[2], buf[3]]),
			i16::from_be_bytes([buf[4], buf[5]])
		))
	}

	pub fn set_averaged_samples(&mut self, amt: AveragedSamples) -> Result<(), E> {
		let reg = self.read_reg(Registers::ConA as u8)?;
		let new = (reg & !AVG_MASK) & ((amt as u8) << 5);
		self.i2c.write(IIC_ADDR, &[Registers::ConA as u8, new])
	}

	pub fn set_output_data_rate(&mut self, rate: OutputRates) -> Result<(), E> {
		let reg = self.read_reg(Registers::ConA as u8)?;
		let new = (reg & !ODR_MASK) & ((rate as u8) << 2);
		self.i2c.write(IIC_ADDR, &[Registers::ConA as u8, new])
	}

	pub fn set_measurement_mode(&mut self, mode: MeasurementModes) -> Result<(), E> {
		let reg = self.read_reg(Registers::ConA as u8)?;
		let new = (reg & !MOD_MASK) & mode as u8;
		self.i2c.write(IIC_ADDR, &[Registers::ConA as u8, new])
	}

	pub fn set_gain(&mut self, gain: Gain) -> Result<(), E> {
		self.i2c.write(
			IIC_ADDR,
			&[Registers::ConB as u8, (gain as u8) << 5],
		)
	}

	pub fn is_locked(&mut self) -> Result<bool, E> {
		let reg = self.read_reg(Registers::Status as u8)?;
		Ok((reg & 0x02) != 0)
	}

	pub fn is_ready(&mut self) -> Result<bool, E> {
		let reg = self.read_reg(Registers::Status as u8)?;
		Ok((reg & 0x01) != 0)
	}

	// returns ascii
	pub fn identify(&mut self) -> Result<[u8; 3], E> {
		let id_a = self.read_reg(Registers::IdA as u8)?;
		let id_b = self.read_reg(Registers::IdB as u8)?;
		let id_c = self.read_reg(Registers::IdC as u8)?;
		Ok([id_a, id_b, id_c])
	}

	// still confused whether this should exist
	pub fn is_hs(&mut self) -> Result<bool, E> {
		let reg = self.read_reg(Registers::Mode as u8)?;
		Ok((!reg & (1 << 7)) != 0)
	}

	pub fn get_output_data_rate(&mut self) -> Result<OutputRates, E> {
		let reg = self.read_reg(Registers::ConA as u8)?;
		let odr = match (reg >> 2) & 0b111 {
			0x00 => OutputRates::Hz0_75,
			0x01 => OutputRates::Hz1_5,
			0x02 => OutputRates::Hz3_0,
			0x03 => OutputRates::Hz7_5,
			0x04 => OutputRates::Hz15_0,
			0x05 => OutputRates::Hz30_0,
			0x06 => OutputRates::Hz75_0,
			_ => panic!(),
		};
		Ok(odr)
	}

	pub fn get_measurement_mode(&mut self) -> Result<MeasurementModes, E> {
		let reg = self.read_reg(Registers::ConA as u8)?;
		let mm = match reg & 0x02 {
			0x00 => MeasurementModes::Normal,
			0x01 => MeasurementModes::PositiveBias,
			0x02 => MeasurementModes::NegativeBias,
			_ => panic!(),
		};
		Ok(mm)
	}

	pub fn get_gain(&mut self) -> Result<Gain, E> {
		let reg = self.read_reg(Registers::ConB as u8)?;
		let gain = match (reg >> 5) & 0b111 {
			0x00 => Gain::_1370,
			0x01 => Gain::_1090,
			0x02 => Gain::_820,
			0x03 => Gain::_660,
			0x04 => Gain::_440,
			0x05 => Gain::_390,
			0x06 => Gain::_330,
			0x07 => Gain::_220,
			_ => panic!(),
		};
		Ok(gain)
	}

}
