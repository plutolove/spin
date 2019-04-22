#![crate_type = "lib"]
#![no_std]


#[cfg(test)]
#[macro_use]
extern crate std;

pub mod mutex;
pub mod rwlock;
