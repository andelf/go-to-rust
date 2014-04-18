#![desc = "A hello world Rust package."]
#![license = "MIT"]

#![crate_id="crypto#crypto:0.1"]
#![crate_type = "lib"]


#![feature(globs, macro_rules, asm)]

extern crate test;
extern crate sync;
extern crate serialize;

pub use crypto::*;


mod crypto;
pub mod hash;


#[cfg(test)]
mod endian_bench {
    use std::io;
    use std::cast;
    use std::mem;
    use std::ptr;
    use test::Bencher;

    // dummy, just want some raw bytes
    static TestSource: &'static [u8] = include_bin!("./lib.rs");

    #[bench]
    fn bench_byte_shift(b: &mut Bencher) {
        let mut w = [0u32, ..100];
        let p = TestSource.clone();
        b.iter(|| {
            for i in range(0u, 100u) {
                let j = i * 4;
                w[i] = p[j] as u32 << 24 | p[j+1] as u32 << 16 | p[j+2] as u32 << 8 | p[j+3] as u32;
            }
        });
    }

    #[bench]
    fn bench_byte_io_ext_call(b: &mut Bencher) {
        let mut w = [0u32, ..100];
        let p = TestSource.clone();
        b.iter(|| {
            for i in range(0u, 100u) {
                let j = i * 4;
                w[i] = io::extensions::u64_from_be_bytes(p, j, 4) as u32;
            }
        });
    }

    // fastest
    #[bench]
    fn bench_byte_mem_to_be(b: &mut Bencher) {
        let mut w = [0u32, ..100];
        let p = TestSource.clone();
        b.iter(|| {
            unsafe {
                ptr::copy_memory(cast::transmute::<_,*mut u8>(&w), p.as_ptr(), 400);
            }
            // FIX ENDIAN
            for i in range(0, 16u) {
                w[i] = mem::to_be32(w[i]);
            }
        });
    }

}
