use serialize::hex::ToHex;

pub trait Hash : Writer {
    fn reset(&mut self);
    fn digest(&self) -> ~[u8];
    fn size(&self) -> uint;
    fn block_size(&self) -> uint;

    fn hexdigest(&self) -> ~str {
        self.digest().to_hex()
    }
}


pub mod adler32;
pub mod crc32;
pub mod crc64;
pub mod fnv;
