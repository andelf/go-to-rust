use std::mem;


pub enum Sha2Varient {
    Sha_224,
    Sha_256,
    Sha_384,
    Sha_512,
    Sha_512_224,
    Sha_512_256,
}


pub trait DigestState {
    fn reset(&mut self);
    fn process_block(&mut self, data: &[u8]);

    fn new() -> Self {
        let mut ret = unsafe { mem::init::<Self>() };
        ret.reset();
        ret
    }
}

pub trait Digester {

}
