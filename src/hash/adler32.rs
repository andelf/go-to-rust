use std::io::IoResult;
use std::cast;
use super::*;


static pmod : u32 = 65521;
static nmax : uint = 5552;

pub static Size : uint = 4;

#[deriving(Eq, Clone, Show)]
pub struct Adler32(u32);

fn update(d: Adler32, p: &[u8]) -> Adler32 {
    let d = unsafe { cast::transmute::<_,u32>(d) };
    let (mut s1, mut s2) = (d & 0xFFFFu32, d >> 16);
    for q in p.chunks(nmax) {
        for x in q.iter() {
            s1 += *x as u32;
            s2 += s1;
        }
        s1 %= pmod;
        s2 %= pmod;
    }
    Adler32(s2 << 16 | s1)
}

impl Adler32 {
    pub fn new() -> Adler32 {
        Adler32(1u32)
    }
}

impl Writer for Adler32 {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        let d = update(*self, buf);
        *self = unsafe { cast::transmute(d) };
        Ok(())
    }
}

impl Hash for Adler32 {
    fn reset(&mut self) {
        *self = Adler32(1);
    }
    fn digest(&self) -> ~[u8] {
        let s = unsafe { cast::transmute::<_, u32>(*self) };
        ~[(s>>24) as u8, (s>>16) as u8, (s>>8) as u8, s as u8]
    }
    fn size(&self) -> uint {
        Size
    }
    fn block_size(&self) -> uint {
        1
    }
}

pub fn checksum(data: &[u8]) -> u32 {
    let d = update(Adler32(1), data);
    unsafe { cast::transmute(d) }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(checksum(bytes!("welcome to china")), 884934163u32);
    }

    #[test]
    fn test_adler32() {
        let mut h = Adler32::new();
        assert!(h.write(bytes!("welcome to china")).is_ok());
        assert_eq!(h.digest().as_slice(), &[52, 191, 6, 19]);
        h.reset();
        //assert_eq!(h.sum32(), 1);
    }
}
