env $(cat .env) cargo build --release
mkdir -p ./target/uf2
elf2uf2-rs target/thumbv6m-none-eabi/release/pico_sensors ./target/uf2/build.uf2