#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{
    color::Color,
    epd2in9_v2::{Display2in9, Epd2in9},
    prelude::*,
};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "the e-paper frame buffer (~4.7 KB) lives in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut delay = Delay::new();

    let cs = Output::new(peripherals.GPIO2, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO3, Level::Low, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let busy = Input::new(peripherals.GPIO4, InputConfig::default());

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7);

    let mut spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let mut epd = Epd2in9::new(&mut spi_device, busy, dc, rst, &mut delay, None).unwrap();

    // In-memory frame buffer: draw here, then push it to the panel in one go
    let mut display = Display2in9::default();
    // Landscape: 296 x 128
    display.set_rotation(DisplayRotation::Rotate90);
    display.clear(Color::White).unwrap();

    Rectangle::new(Point::new(2, 2), Size::new(292, 124))
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 2))
        .draw(&mut display)
        .unwrap();

    let title = MonoTextStyle::new(&FONT_10X20, Color::Black);
    Text::with_alignment("Title", Point::new(148, 35), title, Alignment::Center)
        .draw(&mut display)
        .unwrap();

    Circle::new(Point::new(128, 70), 40)
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 3))
        .draw(&mut display)
        .unwrap();

    epd.update_and_display_frame(&mut spi_device, display.buffer(), &mut delay)
        .unwrap();

    epd.sleep(&mut spi_device, &mut delay).unwrap();

    loop {
        delay.delay_millis(1000);
    }
}