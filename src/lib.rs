//! Core library for the [tlenek](https://github.com/maxgmr/tlenek) x86_64 operating system.
#![warn(missing_docs)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(test_framework::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod qemu;
pub mod serial;
pub mod test_framework;
pub mod vga_text;
