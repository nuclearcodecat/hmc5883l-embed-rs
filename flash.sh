arm-none-eabi-objcopy target/thumbv7m-none-eabi/debug/hmc5883l target/thumbv7m-none-eabi/debug/hmc5883l.bin
st-flash --reset write target/thumbv7m-none-eabi/debug/hmc5883l.bin 0x08000000
