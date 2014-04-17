use std::io::IoResult;
//use std::rc::Rc;
//use sync::one::{Once, ONCE_INIT};
use super::*;

pub static Size : uint = 8;

pub type Table = [u64, ..256];

#[repr(C)]
pub enum Crc64Polynomial {
    ISO,
    ECMA,
    /// Whatever...
    Custom(u64),
}

impl Crc64Polynomial {
    fn as_u64(&self) -> u64 {
        match *self {
            ISO => 0xD800000000000000,
            ECMA => 0xC96C5795D7870F42,
            Custom(p) => p
        }
    }

    fn make_table_into(&self, t: &mut Table) {
        let poly = self.as_u64();
        for i in range(0u64, 256u64) {
            let mut crc = i;
            for _j in range(0, 8) {
                if crc & 1 == 1 {
                    crc = (crc >> 1) ^ poly
                } else {
                    crc >>= 1
                }
            }
            t[i as uint] = crc
        }
    }

    fn make_table(&self) -> Table {
        let mut t : Table = [0u64, ..256];
        self.make_table_into(&mut t);
        t
    }
}


// TODO: use static tables
// static mut castagnoliTable: &'static Table = &'static [0u64, ..256];
// static mut castagnoliOnce: Once = ONCE_INIT;

// #[deriving(Eq, Clone)]
pub struct Crc64 {
    crc: u64,
    tab: ~Table,
}

impl Crc64 {
    pub fn new(poly: Crc64Polynomial) -> Crc64 {
        Crc64 { crc: 0,
                tab: ~poly.make_table()
        }
    }
}

fn update(crc: u64, tab: &Table, p: &[u8]) -> u64 {
    let mut crc = !crc;
    for v in p.iter() {
        crc = tab[(crc as u8 ^ *v) as uint] ^ (crc >> 8)
    }
    return !crc
}



impl Writer for Crc64 {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.crc = update(self.crc, self.tab, buf);
        Ok(())
    }
}

impl Hash for Crc64 {
    fn reset(&mut self) {
        self.crc = 0
    }
    fn digest(&self) -> ~[u8] {
        let s = self.crc;
        ~[(s>>56) as u8, (s>>48) as u8, (s>>40) as u8, (s>>32) as u8,
          (s>>24) as u8, (s>>16) as u8, (s>>8) as u8, s as u8]
    }
    fn size(&self) -> uint {
        Size
    }
    fn block_size(&self) -> uint {
        1
    }
}

pub fn checksum(data: &[u8], tab: &Table) -> u64 {
    update(0, tab, data)
}

#[test]
fn test_update() {
    assert_eq!(1747484016367373810u64, update(0u64, &ISO.make_table(), bytes!("welcome to china")));
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    #[test]
    fn test_crc64() {
        let mut h = Crc64::new(ISO);
        assert!(h.write(bytes!("welcome to china")).is_ok());
        assert_eq!(h.digest().as_slice(), &[24, 64, 79, 116, 78, 64, 13, 242]);
        h.reset();
        //assert_eq!(h.sum64(), 0);
    }


}
