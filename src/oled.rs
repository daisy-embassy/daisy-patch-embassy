use defmt::debug;
use embassy_stm32::gpio::Output;
use embedded_graphics::Drawable;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, Ssd1306Async};

use crate::dma_spi::DmaSpi;

#[embassy_executor::task]
pub async fn oled_task(spi: DmaSpi<'static>, dc: Output<'static>, mut rst: Output<'static>) {
    debug!("oled_task");
    let interface = SPIInterface::new(spi, dc);
    let mut display_spi = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    debug!("reset display");
    display_spi
        .reset(&mut rst, &mut embassy_time::Delay {})
        .await
        .expect("reset failed");
    debug!("init display");
    display_spi.init().await.unwrap();
    let fc = BinaryColor::On;
    let bg = BinaryColor::Off;
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(fc)
        .background_color(bg)
        .build();

    let _ = Text::with_baseline(
        "Hello daisy-patch-embassy!!",
        Point::new(0, 0),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display_spi);

    debug!("flush display");
    display_spi.flush().await.unwrap();
    debug!("oled_task done");
}
