// todo interrupt example

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use defmt::println;
use defmt_rtt as _;
use panic_rtt_target as _;

use hmc5883l::Hmc5883l;
use hmc5883l_embed as hmc5883l;
use stm32f1xx_hal::{
	i2c::{BlockingI2c, Mode},
	pac,
	prelude::*,
};

#[entry]
fn main() -> ! {
	let p = pac::Peripherals::take().unwrap();

	let mut flash = p.FLASH.constrain();
	let rcc = p.RCC.constrain();
	let mut afio = p.AFIO.constrain();
	let clocks = rcc
		.cfgr
		.use_hse(8.MHz())
		.sysclk(72.MHz())
		.freeze(&mut flash.acr);

	let mut gpiob = p.GPIOB.split();
	let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
	let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

	// fast mode i2c on pb6, pb7
	let i2c = BlockingI2c::i2c1(
		p.I2C1,
		(scl, sda),
		&mut afio.mapr,
		Mode::Fast {
			frequency: 400.kHz(),
			duty_cycle: stm32f1xx_hal::i2c::DutyCycle::Ratio2to1,
		},
		clocks,
		1000,
		10,
		1000,
		1000,
	);

	// make a new sensor
	let mut sensor = Hmc5883l::new(i2c);
	// get repeated measurements instead of default, single measurement
	sensor
		.set_operating_mode(hmc5883l::OperationModes::Continuous)
		.unwrap();
	loop {
		// wait 67ms before the next read
		delay(4824000);
		// get the angles
		let (x, y, z) = sensor.get_angles().unwrap();
		println!("x: {}, y: {}, z: {}", x, y, z);
	}
}
