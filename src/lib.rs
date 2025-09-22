#![no_std]
#![no_main]
#![feature(f16)]
#![deny(clippy::mem_forget)]

pub mod app;

pub mod button;

pub mod compass;

pub mod display;

pub mod gps;

pub mod led_ring;

pub mod qmc5883l;

pub mod landmark;

pub mod user_interface;

pub mod generated {
    include! {concat!(env!("OUT_DIR"), "/generated_config.rs")}
}
