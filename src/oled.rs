use embassy_stm32::{gpio::Output, mode::Async, spi::Spi};
use embedded_graphics::Drawable;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, Ssd1306Async};

#[embassy_executor::task]
pub async fn oled_task(
    spi: Spi<'static, Async>,
    cs: Output<'static>,
    dc: Output<'static>,
    mut rst: Output<'static>,
) {
    let spi = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let interface = SPIInterface::new(spi, dc);
    let mut display_i2c = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display_i2c
        .reset(&mut rst, &mut embassy_time::Delay {})
        .await
        .expect("reset failed");
    let fc = BinaryColor::On;
    let bg = BinaryColor::Off;
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(fc)
        .background_color(bg)
        .build();
    let text_h = text_style.font.character_size.height as i32;

    let _ = Text::with_baseline("Hello!!", Point::new(0, 0), text_style, Baseline::Top)
        .draw(&mut display_i2c);
}
