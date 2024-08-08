//! This example uses the RP Pico W board Wifi chip (cyw43).
//! Connects to specified Wifi network and creates a TCP endpoint on port 1234.

#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

extern crate alloc;

use core::str::from_utf8;

use cyw43_pio::PioSpi;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Stack, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0, USB};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use embedded_alloc::Heap;
use embedded_io_async::Write;
use log::{info, warn};
use net::handle_conn;
use rand::RngCore;
use static_cell::StaticCell; //, panic_probe as _};
                             //use rp2040_panic_usb_boot as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

mod events;
mod net;
mod packets;
mod panic;
mod read;
mod write;

// We use the heap to size packets
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Debug, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    for _ in 0..2 {
        info!(".");
        Timer::after_secs(1).await;
    }
    info!("Launching PicoCraft");
    // TODO: remove logging wait thing, dont know why its needed but it works
    Timer::after_nanos(20000).await;

    let mut rng = RoscRng;

    let fw = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
    //});

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<4>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    loop {
        //control.join_open(WIFI_NETWORK).await;
        match control
            .join_wpa2(env!("WIFI_NETWORK"), env!("WIFI_PASSWORD"))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    info!(
        "DHCP is up with IP {}",
        stack.config_v4().unwrap().address.address()
    );
    //Timer::after_millis(100).await;

    // And now we can use it!

    info!("Created bufs");
    //Timer::after_millis(100).await;

    static mut RX_BUF: [[u8; 1024]; 4] = [[0; 1024]; 4];
    static mut TX_BUF: [[u8; 1024]; 4] = [[0; 1024]; 4];

    let mut i = 0;

    loop {
        let rx_buffer = unsafe { &mut RX_BUF[i] };
        let tx_buffer = unsafe { &mut TX_BUF[i] };

        rx_buffer.fill(0);
        rx_buffer.fill(0);

        let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        control.gpio_set(0, false).await;
        info!("Listening on TCP:25565...");
        //Timer::after_millis(100).await;
        if let Err(e) = socket.accept(25565).await {
            warn!("accept error: {:?}", e);
            //Timer::after_millis(100).await;
            continue;
        }

        control.gpio_set(0, true).await;
        info!("Received connection from {:?}", socket.remote_endpoint());
        //Timer::after_millis(100).await;

        spawner.spawn(handle_conn(socket)).unwrap();

        info!("Creating a new thingy majigy");
        //Timer::after_millis(100).await;

        i += 1;
        if i >= 4 {
            i = 0;
        }
    }
}
