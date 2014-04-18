use std::io::IoResult;
use std::mem;
use std::ptr;
use std::cast;
use std::slice;
use super::*;
use super::super::hash::*;

pub static Size: uint = 16;
pub static BlockSize: uint = 64;

static CHUNK: uint = 64;
static init0: u32 = 0x67452301;
static init1: u32 = 0xEFCDAB89;
static init2: u32 = 0x98BADCFE;
static init3: u32 = 0x10325476;

static PerRoundShift: [u8, ..64] = [7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,
                                    5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,
                                    4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,
                                    6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21, ];
// for i in range(0, 64u) { K[i] = ((i as f64 + 1.).sin().abs() * pow(2f64, 32)).floor() as u32 }
static K: [u32, ..64] = [0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
                         0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
                         0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
                         0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
                         0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
                         0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
                         0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
                         0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
                         0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
                         0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
                         0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
                         0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
                         0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
                         0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
                         0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
                         0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391, ];


#[deriving(Eq, Show)]
pub struct Md5 {
    h: (u32, u32, u32, u32),
    x: Vec<u8>,
    len: uint,
}

// deep clone
impl Clone for Md5 {
    fn clone(&self) -> Md5 {
        Md5 {
            h: self.h,
            x: self.x.clone(),
            len: self.len,
        }
    }
}

impl Md5 {
    pub fn new() -> Md5 {
        let mut ret = Md5 { h: (0,0,0,0),
                            x: Vec::new(),
                            len: 0 };
        ret.reset();
        ret
    }

    fn block(&mut self, buf: &[u8]) {
        assert!(buf.len() % CHUNK == 0);
        let mut w = [0u32, ..16]; // 0-15, 16-79
        let (mut h0, mut h1, mut h2, mut h3) = self.h;
        // Process the message in successive 512-bit chunks:
        for p in buf.chunks(CHUNK) {
            //for i in range(0u, 16u) {
            //  let j = i * 4;
            // w[i] = p[j] as u32 << 24 | p[j+1] as u32 << 16 | p[j+2] as u32 << 8 | p[j+3] as u32;
            // md5 mem layout is LE
            unsafe {
                ptr::copy_memory(cast::transmute::<_,*mut u8>(&w), p.as_ptr(), CHUNK);
            }
            // FIX ENDIAN
            for i in range(0, 16u) {
                w[i] = mem::to_le32(w[i]);
            }

            let (mut a, mut b, mut c, mut d) = (h0, h1, h2, h3);
            for i in range(0, 64u) {
                let (f, g) = match i / 16 {
                    0 => ((b & c) | ((! b) & d),
                          i),
                    1 => ((d & b) | ((! d) & c),
                          (5*i + 1) % 16),
                    2 => (b ^ c ^ d,
                          (3*i + 5) % 16),
                    3 => (c ^ (b | (! d)),
                          (7*i) % 16),
                    _ => unreachable!(),
                };
                let tempd = d;
                d = c;
                c = b;
                let mut tempb = a + f + K[i] + w[g];
                unsafe { asm!("movb $2, %cl
                               movl $0, %eax
                               roll %cl, %eax
                               movl %eax, $0"
                              : "=r"(tempb)
                              : "0"(tempb), "r"(PerRoundShift[i])
                              : "rcx", "rax"
                              ) };
                b += tempb;
                a = tempd
            }
            h0 += a;
            h1 += b;
            h2 += c;
            h3 += d;
        }
        self.h = (h0, h1, h2, h3);

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
        //
        let len = (len << 3) as u64;
        for i in range(0, 8u) {
            tmp[i] = (len >> 8*i) as u8;
        }
        self.write(tmp.slice_to(8));
        assert_eq!(self.x.len(), 0);
        let (h0, h1, h2, h3) = self.h;

        let mut ret = slice::with_capacity::<u8>(Size);
        unsafe {
            {
                let p = cast::transmute::<_,*mut u32>(ret.as_mut_ptr());
                slice::raw::mut_buf_as_slice(p, Size / 4, |v| {
                    v[0] = mem::to_le32(h0);
                    v[1] = mem::to_le32(h1);
                    v[2] = mem::to_le32(h2);
                    v[3] = mem::to_le32(h3);
                });
            }
            ret.set_len(Size);
        }
        ret
    }

}

impl Writer for Md5 {
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

impl Hash for Md5 {
    fn reset(&mut self) {
        self.h = (init0, init1, init2, init3);
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
fn test_md5() {
    let mut h = Md5::new();
    assert_eq!(h.hexdigest(), ~"d41d8cd98f00b204e9800998ecf8427e");
    h.write_str("a");
    assert_eq!(h.hexdigest(), ~"0cc175b9c0f1b6a831c399e269772661");
}

#[test]
fn test_md5_long() {
    let mut h = Md5::new();
    h.write_str("welcome to china".repeat(10001));
    assert_eq!(h.hexdigest(), ~"91fdcbaaf79739a20635cd24dd67e532");
}
