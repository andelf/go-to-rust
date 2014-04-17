use std::io::IoResult;
//use std::rc::Rc;
//use sync::one::{Once, ONCE_INIT};
use super::*;


pub static Size : uint = 4;

pub type Table = [u32, ..256];

/// Predefined polynomials.
#[repr(C)]
pub enum Crc32Polynomial {
    /// Far and away the most common CRC-32 polynomial.
    /// Used by ethernet (IEEE 802.3), v.42, fddi, gzip, zip, png, mpeg-2, ...
    IEEE,
    /// Castagnoli's polynomial, used in iSCSI.
    Castagnoli,
    /// Koopman's polynomial.
    Koopman,
    /// Whatever...
    Custom(u32),
}

impl Crc32Polynomial {
    fn as_u32(&self) -> u32 {
        match *self {
            IEEE => 0xedb88320,
            Castagnoli => 0x82f63b78,
            Koopman => 0xeb31d82e,
            Custom(p) => p
        }
    }

    fn make_table_into(&self, t: &mut Table) {
        let poly = self.as_u32();
        for i in range(0u32, 256u32) {
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
        let mut t : Table = [0u32, ..256];
        self.make_table_into(&mut t);
        t
    }
}


// TODO: use static tables
// static mut castagnoliTable: &'static Table = &'static [0u32, ..256];
// static mut castagnoliOnce: Once = ONCE_INIT;

// #[deriving(Eq, Clone)]
pub struct Crc32 {
    crc: u32,
    tab: ~Table,
}

impl Crc32 {
    pub fn new(poly: Crc32Polynomial) -> Crc32 {
        Crc32 { crc: 0,
                tab: ~poly.make_table()
        }
    }

    pub fn newIEEE() -> Crc32 {
        Crc32::new(IEEE)
    }
}

fn update(crc: u32, tab: &Table, p: &[u8]) -> u32 {
    let mut crc = !crc;
    for v in p.iter() {
        crc = tab[(crc as u8 ^ *v) as uint] ^ (crc >> 8)
    }
    return !crc
}



impl Writer for Crc32 {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.crc = update(self.crc, self.tab, buf);
        Ok(())
    }
}

impl Hash for Crc32 {
    fn reset(&mut self) {
        self.crc = 0
    }
    fn digest(&self) -> ~[u8] {
        let s = self.crc;
        ~[(s>>24) as u8, (s>>16) as u8, (s>>8) as u8, s as u8]
    }
    fn size(&self) -> uint {
        Size
    }
    fn block_size(&self) -> uint {
        1
    }
}

pub fn checksum(data: &[u8], tab: &Table) -> u32 {
    update(0, tab, data)
}

pub fn checksumIEEE(data: &[u8]) -> u32 {
    update(0, &IEEE.make_table(), data)
}

#[test]
fn test_update() {
    assert_eq!(716219773u32, update(0u32, &IEEE.make_table(), bytes!("welcome to china")));
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    #[test]
    fn test_checksum() {
        assert_eq!(checksumIEEE(bytes!("welcome to china")), 716219773u32);
    }

    #[test]
    fn test_crc32() {
        let mut h = Crc32::new(IEEE);
        assert!(h.write(bytes!("welcome to china")).is_ok());
        assert_eq!(h.digest().as_slice(), &[42, 176, 165, 125]);
        h.reset();
        //assert_eq!(h.sum32(), 0);
    }


}
