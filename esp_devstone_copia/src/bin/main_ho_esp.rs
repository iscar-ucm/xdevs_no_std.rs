#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::time::Duration;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

use xdevs_devstone::common::*;
use xdevs_devstone::ho::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    // TODO: Spawn some tasks
    let _ = spawner;

    const WIDHT: usize = 2;
    const W: usize = WIDHT - 1;

    let start = embassy_time::Instant::now();
    xdevs_devstone_macros::generate_hi!(2, 2);
    let generator = Generator::new(5);
    let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_ho);
    let duration: Duration = start.elapsed();
    info!("Model creation time: {:?}", duration);
    let start = embassy_time::Instant::now();
    let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
    let config = xdevs::simulator::Config::new(
        embassy_time::Instant::from_secs(0),
        embassy_time::Instant::from_secs(10),
        1,
        None,
    );
    let duration = start.elapsed();
    info!("Simulator creation time: {:?}", duration);
    let start = embassy_time::Instant::now();
    // simulator.simulate_vt(&config);
    let input_handler = xdevs::simulator::SleepAsync::new();
    simulator.simulate_rt_async(&config, input_handler, |_| {});
    let duration = start.elapsed();
    info!("Simulation time: {:?}", duration);

    loop {
        Timer::after(embassy_time::Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
