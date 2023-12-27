#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]


use panic_rtt_target as _;
use rtt_target::rtt_init_print;
// use rtt_target::rprintln;
use stm32f1xx_hal as hal;
use hal:: {
        prelude::*,
        gpio::*,
        spi::*,
    };
use rtic_monotonics::systick::*;
use rtic_sync::{channel::*, make_channel};
use tinyrand::{StdRand, RandRange, Seeded};
use ws2812_blocking_spi::Ws2812BlockingWriter;

use garland::garland::{no_pastel, LED_NUMBER, ColorFrame, SmartLedsWrite, RGB8};


#[rtic::app(device = hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {

    use super::*;


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: ErasedPin<Output>,
        led_strip: Ws2812BlockingWriter<Spi<hal::pac::SPI1, Spi1Remap, (NoSck, NoMiso, Pin<'B', 5, Alternate>), u8>>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {

        rtt_init_print!();

        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain();
 
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(72.MHz())
            .pclk1(36.MHz())
            .pclk2(72.MHz())
            .freeze(&mut flash.acr);

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, clocks.sysclk().to_Hz(), systick_token);

        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase();

        let spi = Spi::spi1(
            cx.device.SPI1,
            (NoSck, NoMiso, gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl)),
            &mut afio.mapr,
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            3.MHz(),
            clocks
        );
        let led_strip = Ws2812BlockingWriter::new(spi);


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
               led_strip
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
    #[task(priority = 1)]
    async fn get_new_color(
        cx: get_new_color::Context,
        mut color_sender: Sender<'static, RGB8, 1>)
    {

        let amplitude = 10u16;
        let mut rand = StdRand::seed(42);

        loop {
            
            let color = RGB8 {
                r: rand.next_range(0..amplitude) as u8,
                g: rand.next_range(0..amplitude) as u8,
                b: rand.next_range(0..amplitude) as u8,
            };
            let color = no_pastel(color);

            color_sender.send(color).await.ok();
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
