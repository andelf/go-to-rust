use std::io::IoResult;
use std::mem;
use std::ptr;
use std::cast;
use std::slice;
use super::*;
use super::super::hash::*;

pub static Size: uint = 20;
pub static BlockSize: uint = 64;


static CHUNK: uint = 64;
static init0: u32 = 0x67452301;
static init1: u32 = 0xEFCDAB89;
static init2: u32 = 0x98BADCFE;
static init3: u32 = 0x10325476;
static init4: u32 = 0xC3D2E1F0;


static _K0 : u32 = 0x5A827999;
static _K1 : u32 = 0x6ED9EBA1;
static _K2 : u32 = 0x8F1BBCDC;
static _K3 : u32 = 0xCA62C1D6;


#[deriving(Eq, Show)]
pub struct Sha1 {
    h: (u32, u32, u32, u32, u32),
    x: Vec<u8>,
    len: uint,
}

// deep clone
impl Clone for Sha1 {
    fn clone(&self) -> Sha1 {
        Sha1 {
            h: self.h,
            x: self.x.clone(),
            len: self.len,
        }
    }
}

impl Sha1 {
    pub fn new() -> Sha1 {
        let mut ret = Sha1 { h: (0,0,0,0,0),
                             x: Vec::new(),
                             len: 0 };
        ret.reset();
        ret
    }

    fn block(&mut self, buf: &[u8]) {
        assert!(buf.len() % CHUNK == 0);
        let mut w = [0u32, ..80]; // 0-15, 16-79
        let (mut h0, mut h1, mut h2, mut h3, mut h4) = self.h;
        for p in buf.chunks(CHUNK) {
            // break chunk into sixteen 32-bit big-endian words w[i], 0 ≤ i ≤ 15
            for i in range(0u, 16u) {
                let j = i * 4;
                w[i] = p[j] as u32 << 24 | p[j+1] as u32 << 16 | p[j+2] as u32 << 8 | p[j+3] as u32;
            }
            // Extend the sixteen 32-bit words into eighty 32-bit words:
            for i in range(16u, 80u) {
                let mut t = w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16];
                unsafe { asm!("roll $0" : "=r"(t) : "0"(t) ) };
                w[i] = t;
            }
            // Initialize hash value for this chunk:
            let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
            // Main loop
            for i in range(0, 80u) {
                let (f, k) : (u32, u32) = if 0 <= i && i <= 19 {
                    ((b & c) | ((! b) & d), _K0)
                } else if 20 <= i && i <= 39 {
                    (b ^ c ^ d, _K1)
                } else if 40 <= i && i <= 59 {
                    ((b & c) | (b & d) | (c & d), _K2)
                } else if 60 <= i && i <= 79 {
                    (b ^ c ^ d, _K3)
                } else {
                    unreachable!()
                };
                let mut temp = a;
                unsafe { asm!("roll $$5, $0" : "=r"(temp) : "0"(temp) ) };
                temp += f + e + k + w[i];
                e = d;
                d = c;
                unsafe { asm!("roll $$30, $1\n movl $1, $0" : "=r"(c) : "r"(b) ) };
                b = a;
                a = temp;
            }
            // Add this chunk's hash to result so far:
            h0 = h0 + a;
            h1 = h1 + b;
            h2 = h2 + c;
            h3 = h3 + d;
            h4 = h4 + e;
        }
        self.h = (h0, h1, h2, h3, h4);
    }

     fn checksum(&mut self) -> ~[u8] {
         let mut tmp = slice::from_elem(64, 0u8);
         tmp[0] = 0x80;
         let mut len = self.len;
         if len % 64 < 56 {
             self.write(tmp.slice_to(56 - len % 64));
         } else {
             self.write(tmp.slice_to(64 + 56 - len % 64));
         }
         len <<= 3;
         for i in range(0, 8u) {
             tmp[i] = (len >> (56 - 8*i)) as u8;
         }
         self.write(tmp.slice_to(8));
         assert_eq!(self.x.len(), 0);
         println!("checksum => {:?}", self);
         let (h0, h1, h2, h3, h4) = self.h;

         let mut ret = slice::with_capacity::<u8>(Size);
         unsafe {
             {
                 let p = cast::transmute::<_,*mut u32>(ret.as_mut_ptr());
                 slice::raw::mut_buf_as_slice(p, Size / 4, |v| {
                     v[0] = mem::to_be32(h0);
                     v[1] = mem::to_be32(h1);
                     v[2] = mem::to_be32(h2);
                     v[3] = mem::to_be32(h3);
                     v[4] = mem::to_be32(h4);
                 });
             }
             ret.set_len(Size);
         }
         ret
     }
}

impl Writer for Sha1 {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.len += buf.len();  // total len
        self.x.push_all(buf);
        let mut next_x = Vec::new();
        let buf = self.x.clone(); // how to avoid clone?
        for p in buf.as_slice().chunks(CHUNK) {
            if p.len() < CHUNK {
                next_x.push_all(p);
            } else {
                self.block(p);
            }
        }
        self.x = next_x;
        Ok(())
    }
}

impl Hash for Sha1 {
    fn reset(&mut self) {
        self.h = (init0, init1, init2, init3, init4);
        self.x = Vec::new();
        self.len = 0;
    }
    fn digest(&self) -> ~[u8] {
        let mut h = self.clone();
        h.checksum()
    }

    fn size(&self) -> uint {
        Size
    }

    fn block_size(&self) -> uint {
        BlockSize
    }
}

#[test]
fn test_sha1() {
    let mut h = Sha1::new();
    h.write_str(""); //
    assert_eq!(h.hexdigest(), ~"da39a3ee5e6b4b0d3255bfef95601890afd80709");
    //let mut h = Sha1::new();
    h.write_str("we");
    assert_eq!(h.hexdigest(), ~"676e6f35cfc173f73fea9fe27699cf8185397f0c");
}

#[test]
fn test_sha1_long() {
    let mut h = Sha1::new();
    h.write_str("welcome to china".repeat(10001));
    assert_eq!(h.hexdigest(), ~"da3884df7c84378ebf72b86e3fe43b2a4664d73a");
}
