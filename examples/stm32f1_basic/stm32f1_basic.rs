#![allow(unsafe_code)]
// needed for rust 2024 to allow static mut refs and avoid a lot of painful types
#![allow(static_mut_refs)]
#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;
use panic_rtt_target as _;

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use hmc5883l::Hmc5883l;
use hmc5883l_embed as hmc5883l;
use pac::interrupt;
use stm32f1xx_hal::{
	gpio::{Alternate, ExtiPin, Input, OpenDrain, PullDown, PB6, PB7},
	i2c::{BlockingI2c, Mode},
	pac,
	prelude::*,
};

#[entry]
fn main() -> ! {
	info!("starting");
	let mut p = pac::Peripherals::take().unwrap();

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

	{
		// prepare pin for interrupts
		let mut gpioa = p.GPIOA.split();
		let mut exti_pin = gpioa.pa7.into_pull_down_input(&mut gpioa.crl);
		exti_pin.make_interrupt_source(&mut afio);
		exti_pin.trigger_on_edge(&mut p.EXTI, stm32f1xx_hal::gpio::Edge::Rising);
		exti_pin.enable_interrupt(&mut p.EXTI);

		// make a new sensor
		let mut sensor = Hmc5883l::new(i2c);
		// get repeated measurements instead of default, single measurement
		// this will fail first if there is any i2c error - misconnected wires, sensor not powered, etc.
		sensor.set_operating_mode(hmc5883l_embed::OperationModes::Continuous).map_err(|_| defmt::panic!("failed at first sensor operation")).ok();

		unsafe {
			EXTI_PIN.write(exti_pin);
			SENSOR.write(sensor);
		}
	}

	// enable the interrupt
	unsafe {
		pac::NVIC::unmask(pac::Interrupt::EXTI9_5);
	}

	loop {
	}
}

// honestly i didn't need to write out all of these traits, this could just be a : &mut Hmc5883l<BlockingI2c....
fn print_xyz<I2C, E>(sensor: &mut Hmc5883l<I2C>) -> Result<(), E>
where
	I2C: Write<Error = E> + WriteRead<Error = E> + Read<Error = E>,
{
	// get the angles
	let (x, y, z) = sensor.get_angles()?;
	info!("x: {}, y: {}, z: {}", x, y, z);
	// convert angles to degrees for convenience
	let compass_rad = libm::atan2f(y as f32, x as f32);
	let mut compass_deg = compass_rad.to_degrees();

	if compass_deg < 0.0 {
		 compass_deg += 360.0;
	}

	info!("{}°", compass_deg);
	Ok(())
}

// interrupts need static variables since they don't take any params
static mut EXTI_PIN: MaybeUninit<stm32f1xx_hal::gpio::gpioa::PA7<Input<PullDown>>> = MaybeUninit::uninit();
static mut SENSOR: MaybeUninit<Hmc5883l<BlockingI2c<pac::I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>)>>> = MaybeUninit::uninit();

#[interrupt]
fn EXTI9_5() {
	// get the possibly uninitalized variables
	// they of course are initalized in main, but if they weren't, this would fail
	unsafe {
		let exti_pin = EXTI_PIN.assume_init_mut();
		let sensor = SENSOR.assume_init_mut();
		if exti_pin.check_interrupt() {
			print_xyz(sensor).ok();

			exti_pin.clear_interrupt_pending_bit();
		}
	}
}
