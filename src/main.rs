#![no_std]
#![no_main]

mod dma_spi;
mod oled;
mod usart_midi;

use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::peripherals;
use embassy_stm32::usart;
use embassy_stm32::{
    bind_interrupts,
    gpio::{self, Output},
    spi::Spi,
};
use embassy_time::Timer;
use oled::oled_task;
use usart_midi::midi_task;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let config = daisy_embassy::default_rcc();
    let p = embassy_stm32::init(config);
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

    let dma_spi = dma_spi::DmaSpi::new(spi, cs);

    spawner.must_spawn(oled_task(dma_spi, dc, rst));

    let mut config = usart::Config::default();
    config.baudrate = 32_150; // MIDI baud rate
    let uart = defmt::unwrap!(embassy_stm32::usart::Uart::new(
        p.USART1,
        daisy_p.pins.d14,
        daisy_p.pins.d13,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        config,
    ));
    spawner.must_spawn(midi_task(uart));

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
