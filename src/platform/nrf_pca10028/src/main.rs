//! Tock kernel for the Nordic Semiconductor nRF51 development 
//! kit (DK), a.k.a. the PCA10028. This is an nRF51422 SoC (a
//! Cortex M0 core with a BLE transciver) with many exported
//! pins, LEDs, and buttons. Currently the kernel provides
//! application timers, and GPIO. It will provide a console
//! once the UART is fully implemented and debugged. The
//! application GPIO pins are:
//!
//!   0 -> LED1 (pin 21)
//!   1 -> LED2 (pin 22)
//!   2 -> LED3 (pin 23)
//!   3 -> LED4 (pin 24)
//!   5 -> BUTTON1 (pin 17)
//!   6 -> BUTTON2 (pin 18)
//!   7 -> BUTTON3 (pin 19)
//!   8 -> BUTTON4 (pin 20)
//!   9 -> P0.01   (bottom left header)
//!  10 -> P0.02   (bottom left header)
//!  11 -> P0.03   (bottom left header)
//!  12 -> P0.04   (bottom left header)
//!  12 -> P0.05   (bottom left header)
//!  13 -> P0.06   (bottom left header)
//!  14 -> P0.19   (mid right header)
//!  15 -> P0.18   (mid right header)
//!  16 -> P0.17   (mid right header)
//!  17 -> P0.16   (mid right header)
//!  18 -> P0.15   (mid right header)
//!  19 -> P0.14   (mid right header)
//!  20 -> P0.13   (mid right header)
//!  21 -> P0.12   (mid right header)
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Author: Anderson Lizardo <anderson.lizardo@gmail.com>
//!  Date: August 18, 2016

#![crate_name = "nrf_pca10028"]
#![no_std]
#![no_main]
#![feature(core_intrinsics,lang_items)]

extern crate cortexm0;
extern crate drivers;
extern crate hil;
extern crate nrf51;
extern crate main;
extern crate support;

use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use drivers::timer::TimerDriver;
use nrf51::timer::TimerAlarm;
use nrf51::timer::ALARM1;
use hil::gpio::GPIOPin;
use hil::uart::UART;

// The nRF51 DK LEDs (see back of board)
const LED1_PIN:  usize = 21;
const LED2_PIN:  usize = 22;
const LED3_PIN:  usize = 23;
const LED4_PIN:  usize = 24;

// The nRF51 DK buttons (see back of board)
const BUTTON1_PIN: usize = 17;
const BUTTON2_PIN: usize = 18;
const BUTTON3_PIN: usize = 19;
const BUTTON4_PIN: usize = 20;

pub mod systick;

static mut bytes: [u8; 8] = [0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77];

unsafe fn load_process() -> &'static mut [Option<main::process::Process<'static>>] {
    use core::intrinsics::{volatile_load,volatile_store};
    extern {
        /// Beginning of the ROM region containing app images.
        static _sapps : u8;
    }


    #[link_section = ".app_memory"]
    static mut MEMORY: [u8; 8192] = [0; 8192];
    static mut PROCS: [Option<main::process::Process<'static>>; 1] = [None];

    let addr = &_sapps as *const u8;

    // The first member of the LoadInfo header contains the total size of
    // each process image. A sentinel value of 0 (invalid because it is 
    // smaller than the header itself) is used to mark the end of the list 
    // of processes.
    let total_size = volatile_load(addr as *const usize);
    if total_size != 0 {
        volatile_store(&mut PROCS[0], Some(main::process::Process::create(addr,
                                                                          total_size, &    mut MEMORY)));
    }
    &mut PROCS
}

pub struct Platform {
    gpio: &'static drivers::gpio::GPIO<'static, nrf51::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, TimerAlarm>>,
    console: &'static drivers::console::Console<'static, nrf51::uart::UART>,
}

impl hil::uart::Client for Platform {
    fn read_done(&self, byte: u8) {

    }

    fn write_done(&self, buf: &'static mut[u8]) {
        unsafe {nrf51::uart::UART0.send_bytes(&mut bytes as &'static mut [u8; 8], bytes.len());}
    }
}

impl main::Platform for Platform {
    #[inline(never)]
    fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
        F: FnOnce(Option<&main::Driver>) -> R {
            match driver_num {
                0 => f(Some(self.console)),
                1 => f(Some(self.gpio)),
                3 => f(Some(self.timer)),
                _ => f(None)
            }
        }
}
macro_rules! static_init {
    ($V:ident : $T:ty = $e:expr, $size:expr) => {
        // Ideally we could use mem::size_of<$T> here instead of $size, however
        // that is not currently possible in rust. Instead we write the size as
        // a constant in the code and use compile-time verification to see that
        // we got it right
        let $V : &'static mut $T = {
            use core::{mem, ptr};
            // This is our compile-time assertion. The optimizer should be able
            // to remove it from the generated code.
            let assert_buf: [u8; $size] = mem::uninitialized();
            let assert_val: $T = mem::transmute(assert_buf);
            mem::forget(assert_val);

            // Statically allocate a read-write buffer for the value, write our
            // initial value into it (without dropping the initial zeros) and
            // return a reference to it.
            static mut BUF: [u8; $size] = [0; $size];
            let mut tmp : &mut $T = mem::transmute(&mut BUF);
            ptr::write(tmp as *mut $T, $e);
            tmp
        };
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    nrf51::init();

    static_init!(gpio_pins : [&'static nrf51::gpio::GPIOPin; 22] = [
                 &nrf51::gpio::PORT[LED1_PIN], // 21
                 &nrf51::gpio::PORT[LED2_PIN], // 22
                 &nrf51::gpio::PORT[LED3_PIN], // 23
                 &nrf51::gpio::PORT[LED4_PIN], // 24
                 &nrf51::gpio::PORT[BUTTON1_PIN], // 17
                 &nrf51::gpio::PORT[BUTTON2_PIN], // 18
                 &nrf51::gpio::PORT[BUTTON3_PIN], // 19
                 &nrf51::gpio::PORT[BUTTON4_PIN], // 20
                 &nrf51::gpio::PORT[1],  // Bottom left header on DK board
                 &nrf51::gpio::PORT[2],  //   |
                 &nrf51::gpio::PORT[3],  //   V 
                 &nrf51::gpio::PORT[4],  // 
                 &nrf51::gpio::PORT[5],  //
                 &nrf51::gpio::PORT[6],  // -----
                 &nrf51::gpio::PORT[19], // Mid right header on DK board
                 &nrf51::gpio::PORT[18], //   |
                 &nrf51::gpio::PORT[17], //   V
                 &nrf51::gpio::PORT[16], //  
                 &nrf51::gpio::PORT[15], //  
                 &nrf51::gpio::PORT[14], //  
                 &nrf51::gpio::PORT[13], //  
                 &nrf51::gpio::PORT[12], //  
                 ], 4 * 22);

    static_init!(gpio: drivers::gpio::GPIO<'static, nrf51::gpio::GPIOPin> =
                 drivers::gpio::GPIO::new(gpio_pins), 20);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    static_init!(console: drivers::console::Console<nrf51::uart::UART> =
                 drivers::console::Console::new(&nrf51::uart::UART0, 
                                                &mut drivers::console::WRITE_BUF, 
                                                main::Container::create()),
                                                24);
    nrf51::uart::UART0.set_client(console);

    nrf51::uart::UART0.init(hil::uart::UARTParams {
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None,
        mode: hil::uart::Mode::Normal});
    nrf51::uart::UART0.enable_tx();

//        nrf51::uart::UART0.send_byte(0x55);
//        nrf51::uart::UART0.send_byte(0x8f);
    // The timer driver is built on top of hardware timer 1, which is implemented
    // as an HIL Alarm. Timer 0 has some special functionality for the BLE transciever,
    // so is reserved for that use. This should be rewritten to use the RTC (off the
    // low frequency clock) for lower power.
    let alarm = &nrf51::timer::ALARM1;
    static_init!(mux_alarm : MuxAlarm<'static, TimerAlarm> = MuxAlarm::new(&ALARM1), 16);
    alarm.set_client(mux_alarm);

    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, TimerAlarm> =
                 VirtualMuxAlarm::new(mux_alarm), 24);
    static_init!(timer : TimerDriver<'static, VirtualMuxAlarm<'static,
                 TimerAlarm>> = TimerDriver::new(virtual_alarm1,
                                                 main::Container::create()), 12);
    virtual_alarm1.set_client(timer);
    alarm.enable_nvic();
    alarm.enable_interrupts();

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf51::clock::CLOCK.low_stop();
    nrf51::clock::CLOCK.high_stop();

    nrf51::clock::CLOCK.low_set_source(nrf51::clock::LowClockSource::RC);
    nrf51::clock::CLOCK.low_start();
    nrf51::clock::CLOCK.high_start();
    while !nrf51::clock::CLOCK.low_started() {}
    while !nrf51::clock::CLOCK.high_started() {}

    static_init!(platform: Platform = Platform {
        gpio: gpio,
        timer: timer,
        console: console,
    }, 12);

    alarm.start();

    // The systick implementation currently directly accesses the low clock
    // when it configures the real time clock (RTC); it should go through 
    // clock::CLOCK instead.
    systick::reset();
    systick::enable(true); 
    nrf51::uart::UART0.send_bytes(&mut bytes as &'static mut [u8; 8], 8);
    nrf51::gpio::PORT[LED1_PIN].toggle();
    main::main(platform, &mut nrf51::chip::NRF51::new(), load_process());

}


use core::fmt::Arguments;
#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern fn rust_begin_unwind(_args: &Arguments,
                                       _file: &'static str, _line: usize) -> ! {
    use support::nop;
    use hil::gpio::GPIOPin;

    let led0 = &nrf51::gpio::PORT[LED1_PIN];
    let led1 = &nrf51::gpio::PORT[LED2_PIN];

    led0.enable_output();
    led1.enable_output();
    loop {
        for _ in 0..100000 {
            led0.set();
            led1.set();
            nop();
        }
        for _ in 0..100000 {
            led0.clear();
            led1.clear();
            nop();
        }
    }
}
