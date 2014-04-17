use std::io::IoResult;
use std::ops::{BitXor, Mul};
use std::num::FromPrimitive;
use std::slice;
use std::ptr;
use std::cast;
use std::mem;
use super::*;



pub trait FnvHasher : FromPrimitive {
    fn prime() -> Self;
    fn offset() -> Self;
}

impl FnvHasher for u32 {
    fn prime() -> u32 { 16777619 }
    fn offset() -> u32 { 2166136261 }
}

impl FnvHasher for u64 {
    fn prime() -> u64 { 1099511628211 }
    fn offset() -> u64 { 14695981039346656037 }
}


pub struct Fnv1<T>(T);

impl<T: FnvHasher + Num> Fnv1<T> {
    fn new() -> Fnv1<T> {
        Fnv1(FnvHasher::offset())
    }
}

impl<T: FnvHasher + BitXor<T,T> + Mul<T,T>> Writer for Fnv1<T> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        let Fnv1(ref mut h) = *self;
        for c in buf.iter() {
            *h = h.mul(&FnvHasher::prime());
            *h = h.bitxor(&FromPrimitive::from_u8(*c).expect("can't convert byte to your hash type"))
        }
        Ok(())
    }
}

impl<T: FnvHasher + BitXor<T,T> + Mul<T,T>> Hash for Fnv1<T> {
    fn reset(&mut self) {
        *self = Fnv1(FnvHasher::offset())
    }
    fn digest(&self) -> ~[u8] {
        let size = mem::size_of::<T>();
        let mut ret = slice::with_capacity::<u8>(size);
        unsafe {
            ptr::copy_memory(ret.as_mut_ptr(),
                             cast::transmute(self),
                             size);
            ret.set_len(size);

        }
        ret.into_owned()
    }
    fn size(&self) -> uint {
        mem::size_of::<T>()
    }
    fn block_size(&self) -> uint {
        1
    }
}


// FNV 1a
pub struct Fnv1a<T>(T);

impl<T: FnvHasher + Num> Fnv1a<T> {
    fn new() -> Fnv1a<T> {
        Fnv1a(FnvHasher::offset())
    }
}

impl<T: FnvHasher + BitXor<T,T> + Mul<T,T>> Writer for Fnv1a<T> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        let Fnv1a(ref mut h) = *self;
        for c in buf.iter() {
            *h = h.bitxor(&FromPrimitive::from_u8(*c).expect("can't convert byte to your hash type"));
            *h = h.mul(&FnvHasher::prime());
        }
        Ok(())
    }
}

impl<T: FnvHasher + BitXor<T,T> + Mul<T,T>> Hash for Fnv1a<T> {
    fn reset(&mut self) {
        *self = Fnv1a(FnvHasher::offset())
    }
    fn digest(&self) -> ~[u8] {
        let size = mem::size_of::<T>();
        let mut ret = slice::with_capacity::<u8>(size);
        unsafe {
            ptr::copy_memory(ret.as_mut_ptr(),
                             cast::transmute(self),
                             size);
            ret.set_len(size);

        }
        ret.into_owned()
    }
    fn size(&self) -> uint {
        mem::size_of::<T>()
    }
    fn block_size(&self) -> uint {
        1
    }
}



#[test]
fn test_fnv1_writer() {
    let mut h = Fnv1::<u32>::new();
    h.write(bytes!("welcome to china"));
    assert_eq!(h.hexdigest(), ~"07eb2b33");

}

#[test]
fn test_fnv1a_writer() {
    let mut h = Fnv1a::<u32>::new();
    h.write(bytes!("welcome to china"));
    println!("h => {:?}", h);
    println!("h => {:?}", h.hexdigest());
    assert_eq!(h.hexdigest(), ~"6b4088c1");
}
