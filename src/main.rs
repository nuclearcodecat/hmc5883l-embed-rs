//! This program writes the sensor values to the debug output provided by semihosting
//! you must enable semihosting in gdb with `monitor arm semihosting enable` I have it
//! added to my `.gdbinit`. Then the debug infomation will be printed in your openocd
//! terminal.
//!
//! This program dose not fit on my blue pill unless compiled in release mode
//! eg. `cargo run --example i2c-bme280 --features "stm32f103 bme280 rt" --release`
//! However as noted above the debug output with the read values will be in the openocd
//! terminal.

#![deny(unsafe_code)]
#![no_std]
#![no_main]

mod hmc5883l;

use cortex_m_rt::entry;
use panic_halt as _;

use hmc5883l::Hmc5883l;
use stm32f1xx_hal::{
	i2c::{BlockingI2c, Mode},
	pac,
	prelude::*,
};

#[entry]
fn main() -> ! {
	// Get access to the device specific peripherals from the peripheral access crate
	let p = pac::Peripherals::take().unwrap();

	// Take ownership over the raw flash and rcc devices and convert them into the corresponding
	// HAL structs
	let mut flash = p.FLASH.constrain();
	let rcc = p.RCC.constrain();
	let mut afio = p.AFIO.constrain();
	// Freeze the configuration of all the clocks in the system and store the frozen frequencies in
	// `clocks`
	let clocks = if false == true {
		rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr)
	} else {
		// My blue pill with a stm32f103 clone dose not seem to respect rcc so will not compensate its pulse legths
		// with a faster clock like this. And so the sensor dose not have time to respond to the START pulse.
		// I would be interested if others with real stm32f103's can use this program with the faster clocks.
		rcc.cfgr
			.use_hse(8.MHz())
			.sysclk(48.MHz())
			.pclk1(6.MHz())
			.freeze(&mut flash.acr)
	};

	// Acquire the GPIOB peripheral
	let mut gpiob = p.GPIOB.split();

	let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
	let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

	let i2c = BlockingI2c::i2c1(
		p.I2C1,
      (scl, sda),
		&mut afio.mapr,
		Mode::Fast { frequency: 400.kHz(), duty_cycle: stm32f1xx_hal::i2c::DutyCycle::Ratio2to1 },
		clocks,
		1000,
		10,
		1000,
		1000,
	);

	let mut sens = Hmc5883l::new(i2c);
	loop {}
}
