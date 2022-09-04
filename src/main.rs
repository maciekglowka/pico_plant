#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use core::{
    fmt::Write,
    panic::PanicInfo
};

use embassy_executor::Spawner;
use embassy_rp;
use embassy_rp::{
    gpio::Flex,
};
use embassy_time::{
    Delay,
    Duration,
    Timer,
};

use heapless::String;

use defmt_rtt as _;

mod dht22;
mod soil;
mod wifi;

const READ_INTERVAL: u64 = 3600;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // dht22

    let mut pin_2 = Flex::new(p.PIN_2);
    let mut dht = dht22::DHT22 { pin: pin_2, delay: Delay };

    // wifi 

    let wifi_device = wifi::Wifi::new(
        spawner,
        p.PIN_23,
        p.PIN_25,
        p.PIN_29,
        p.PIN_24,
        env!("WIFI_NAME"),
        env!("WIFI_PASS")
    ).await;

    // soil
    let soil_sensor = soil::SoilSensor::new(0);
    soil_sensor.init();

    async {
        loop {

            loop {
                let dht_reading = match dht.read() {
                    Ok(r) => r,
                    Err(_) => dht22::Reading { temp: 0.0, hum :0.0 }
                };                 
                
                let soil_reading = soil_sensor.read_single();

                let mut s = String::<256>::from("");
                let _ = write!(s, "{{ \"soil_0\": {:.2},  \"temp_0\": {:.2},  \"hum_0\": {:.2} }}", soil_reading, dht_reading.temp, dht_reading.hum );

                wifi_device.connect_and_send(
                    env!("HOST_IP"),
                    env!("HOST_NAME"),
                    env!("HOST_PORT"),
                    env!("HOST_PATH"),
                    s.as_bytes()
                ).await;

                Timer::after(Duration::from_secs(READ_INTERVAL)).await;
            }

        }
    }.await;

}
