// this part is largely copied from the example code of:
// https://github.com/embassy-rs/cyw43

use core::str::FromStr;

use cyw43::NetDevice;
use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_net::{
    Ipv4Address,
    tcp::{ConnectError, TcpSocket},
    Stack,
    StackResources
};
use embassy_rp::{
    gpio::{Flex, Level, Output},
    interrupt,
    peripherals::{PIN_23, PIN_24, PIN_25, PIN_29},
};
use embedded_hal_async::spi::{ExclusiveDevice, SpiBusFlush, SpiBusRead, SpiBusWrite};
use embedded_io::asynch::{Read, Write as Emb_Write};
use heapless::String;
use static_cell::StaticCell;

mod spi;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        STATIC_CELL.init_with(move || $val)
    }};
}

pub struct Wifi {
    stack: &'static Stack<NetDevice<'static>>
}

impl Wifi {
    pub async fn new(
        spawner: Spawner,
        pwr_pin: PIN_23,
        cs_pin: PIN_25,
        clk_pin: PIN_29,
        dio_pin: PIN_24,
        ssid: &str,
        pass: &str
    ) -> Wifi {
        let fw = include_bytes!("../../firmware/43439A0.bin");
        let clm = include_bytes!("../../firmware/43439A0_clm.bin");

        let pwr = Output::new(pwr_pin, Level::Low);
        let cs = Output::new(cs_pin, Level::High);
        let clk = Output::new(clk_pin, Level::Low);
        let mut dio = Flex::new(dio_pin);
        dio.set_low();
        dio.set_as_output();

        let bus = spi::WifiSpi { clk, dio };
        let spi = ExclusiveDevice::new(bus, cs);

        let state = singleton!(cyw43::State::new());
        let (mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

        spawner.spawn(wifi_task(runner)).unwrap();

        let net_device = control.init(clm).await;
        control.join_wpa2(ssid, pass).await;

        let config = embassy_net::ConfigStrategy::Dhcp;

        let seed = 0x0123_4567_89ab_cdef;
        let stack: &Stack<NetDevice> = &*singleton!(
            Stack::new(
                net_device,
                config,
                singleton!(StackResources::<1, 2, 8>::new()),
                seed
            )
        );
        spawner.spawn(net_task(stack)).unwrap();  
        Wifi { stack }      
    }

    pub async fn connect_and_send(
        &self,
        host_ip: &str,
        host_name: &str,
        host_port: &str
    ) {
        // TODO add connection attempts
        let mut rx_buf = [0; 4096];
        let mut tx_buf = [0; 4096];
        let mut buf = [0; 4096];

        let mut socket = TcpSocket::new(self.stack, &mut rx_buf, &mut tx_buf);
        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(5)));

        let addr = Ipv4Address::from_str(host_ip).unwrap();
        let port = host_port.parse::<u16>().unwrap();
        // if let Ok(_) = socket.connect((Ipv4Address::new(192, 168, 1, 104), 8000)).await {
        if let Ok(_) = socket.connect((addr, port)).await {

            let mut host_str = String::<256>::new();
            let _ = write!(host_str, "Host: {}\r\n", host_name);
            
            socket.write_all(b"GET / HTTP/1.1\r\n").await;
            socket.write_all(host_str.as_bytes()).await;
            // socket.write_all(b"Host: 192.168.1.104\r\n").await;
            socket.write_all(b"\r\n").await;

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => (),
                    Err(_) => break
                };
            }
        }
    }
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static, PIN_23>, ExclusiveDevice<spi::WifiSpi, Output<'static, PIN_25>>>
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(
    stack: &'static Stack<cyw43::NetDevice<'static>>) -> ! {
    stack.run().await
}