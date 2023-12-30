#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]


use panic_rtt_target as _;
use rtt_target::rtt_init_print;
// use rtt_target::rprintln;
use rtic_monotonics::systick::*;
use rtic_sync::{channel::*, make_channel};
use tinyrand::{StdRand, RandRange, Seeded};
use ws2812_blocking_spi::Ws2812BlockingWriter;

use garland::bsp::{Board, hal};
use hal:: {
        gpio::*,
        spi::*,
        adc::*,
    };
use garland::garland::{
    no_pastel,
    // single_point,
    triangle_wave,
    LED_NUMBER,
    AMPLITUDE,
    ColorFrame,
    SmartLedsWrite,
    RGB8};
use garland::adc_rand_seed::adc_seed;


#[rtic::app(device = hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {

    use super::*;


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: ErasedPin<Output>,
        led_strip: Ws2812BlockingWriter<Spi<hal::pac::SPI1, Spi1Remap, (NoSck, NoMiso, Pin<'B', 5, Alternate>), u8>>,
        adc_pin: Pin<'A', 0, Analog>,
        adc: Adc<hal::pac::ADC1>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {

        rtt_init_print!();

        let board = Board::new(cx.device);
        let led = board.led;
        let led_strip = Ws2812BlockingWriter::new(board.spi);
        let adc_pin = board.adc_pin;
        let adc = board.adc;

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, board.clocks.sysclk().to_Hz(), systick_token);

        let (color_sender, color_receiver) = make_channel!(RGB8, 1);
        let (frame_sender, frame_receiver) = make_channel!(ColorFrame, 1);

        heartbeat::spawn().ok();
        get_new_color::spawn(color_sender).ok();
        generate_color_frame::spawn(color_receiver, frame_sender).ok();
        update_led_strip::spawn(frame_receiver).ok();

        (
            Shared {
               
            },
            Local {
               led,
               led_strip,
               adc_pin,
               adc
            },
        )
    }

    // Blink on-board LED
    #[task(local = [led], priority = 1)]
    async fn heartbeat(cx: heartbeat::Context) {

        let heartbeat::LocalResources
            {led, ..} = cx.local;

        loop {
            
            led.toggle();

            Systick::delay(1000.millis()).await;
        }
    }

    // Generate random RGB color
    #[task(local = [adc, adc_pin], priority = 1)]
    async fn get_new_color(
        cx: get_new_color::Context,
        mut color_sender: Sender<'static, RGB8, 1>)
    {
        let get_new_color::LocalResources
            {adc, adc_pin, ..} = cx.local;

        let seed = adc_seed(adc, adc_pin).unwrap();
        let mut rand = StdRand::seed(seed as u64);

        loop {
            
            let color = RGB8 {
                r: rand.next_range(0..AMPLITUDE) as u8,
                g: rand.next_range(0..AMPLITUDE) as u8,
                b: rand.next_range(0..AMPLITUDE) as u8,
            };
            let color = no_pastel(color);
            let pattern = triangle_wave(color);

            for color in pattern {

                color_sender.send(color).await.ok();
            }
        }
    }

    /// Generate color frame for LED strip using random color
    #[task(priority = 1)]
    async fn generate_color_frame(
        _cx: generate_color_frame::Context,
        mut color_receiver: Receiver<'static, RGB8, 1>,
        mut frame_sender: Sender<'static, ColorFrame, 1>)
    {

        let mut frame: ColorFrame = [RGB8::default(); LED_NUMBER];

        loop {

            for index in (1..frame.len()).rev() {

                frame[index] = frame[index-1];
            }

            frame[0] = color_receiver.recv().await.unwrap();

            frame_sender.send(frame).await.ok();
            Systick::delay(80.millis()).await;
        }
    }

    // Send color frame to physical LED strip
    #[task(local = [led_strip], priority = 2)]
    async fn update_led_strip(
        cx: update_led_strip::Context, 
        mut frame_receiver: Receiver<'static, ColorFrame, 1>)
    {

        let led_strip = cx.local.led_strip;
        
        loop {

            let frame = frame_receiver.recv().await.unwrap();
            led_strip.write(frame.iter().cloned()).unwrap();
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {

        loop {
            continue;
        }
    }
}
