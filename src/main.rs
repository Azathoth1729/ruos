#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// test attribute
#![feature(custom_test_frameworks)]
#![test_runner(ruos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use ruos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Weclome to RUST OS!");

    ruos::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    ruos::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ruos::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ruos::test_panic_handler(info)
}
