#![no_std]
#![no_main]

mod oled;

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{self, Output},
    spi::Spi,
};
use embassy_time::Timer;
use oled::oled_task;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    let daisy_p = new_daisy_board!(p);
    let spi = Spi::new_txonly(
        p.SPI1,
        daisy_p.pins.d8,
        daisy_p.pins.d10,
        p.DMA1_CH3,
        Default::default(),
    );
    let cs = Output::new(daisy_p.pins.d7, gpio::Level::Low, gpio::Speed::Low);
    let dc = Output::new(daisy_p.pins.d9, gpio::Level::Low, gpio::Speed::Low);
    let rst = Output::new(daisy_p.pins.d30, gpio::Level::Low, gpio::Speed::Low);

    spawner.must_spawn(oled_task(spi, cs, dc, rst));

    let mut led = daisy_p.user_led;

    loop {
        info!("on");
        led.on();
        Timer::after_millis(300).await;

        info!("off");
        led.off();
        Timer::after_millis(300).await;
    }
}
