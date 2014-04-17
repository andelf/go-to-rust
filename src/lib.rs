#![desc = "A hello world Rust package."]
#![license = "MIT"]

#![crate_id="crypto#crypto:0.1"]
#![crate_type = "lib"]


#![feature(globs, macro_rules, asm)]

extern crate sync;
extern crate serialize;

pub use crypto::*;


mod crypto;
pub mod hash;
