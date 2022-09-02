#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use core::{
    fmt::Write,
    panic::PanicInfo
};

// use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp;
use embassy_rp::{
    gpio::Flex,
    interrupt,
    usb::Driver
};
use embassy_time::{
    Delay,
    Duration,
    Timer,
};
use embassy_usb::{
    Builder,
    Config
};
use embassy_usb_serial::{CdcAcmClass, State};

use futures::future::join;
use heapless::String;

use defmt_rtt as _;

mod dht22;
mod soil;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let irq = interrupt::take!(USBCTRL_IRQ);
    let driver = Driver::new(p.USB, irq);

    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("M");
    config.product = Some("piczko W");
    config.serial_number = Some("1234");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
        None
    );

    let mut usb_class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let mut usb = builder.build();

    // dht22

    // let mut pin_2 = Flex::new(p.PIN_2);
    // let mut dht = dht22::DHT22 { pin: pin_2, delay: Delay };

    // soil
    let soil_sensor = soil::SoilSensor::new(0);
    soil_sensor.init();

    join(
        usb.run(),
        async {
            loop {
                usb_class.wait_connection().await;

                loop {
                    usb_class.write_packet(b"Hello\n").await;

                    // if let Ok(reading) = dht.read() {
                    //     let mut s = String::<256>::from("data:");
                    //     let _ = write!(s, "{:?}", reading);
                    //     usb_class.write_packet(s.as_bytes()).await;
                    //     usb_class.write_packet(b"\n").await;
                    // }                    
                    //
                    let bits = soil_sensor.read_single();

                    let mut s = String::<256>::from("data:");
                    let _ = write!(s, "{:.2}", bits);
                    usb_class.write_packet(s.as_bytes()).await;
                    usb_class.write_packet(b"\n").await;

                    Timer::after(Duration::from_secs(2)).await;
                }

            }
        }
    ).await;

}
