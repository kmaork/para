use std::mem::MaybeUninit;
use std::mem;

pub struct Circus<T, const N: usize> {
    arr: [MaybeUninit<T>; N],
    write_idx: usize,
    read_idx: usize,
}

impl<T, const N: usize> Circus<T, N> {
    fn new() -> Self {
        Circus { arr: MaybeUninit::uninit_array(), write_idx: 0, read_idx: 0 }
    }

    #[inline]
    fn read(&mut self) -> T {
        let val = unsafe { mem::replace(&mut self.arr[self.read_idx % N], MaybeUninit::uninit()).assume_init() };
        self.read_idx += 1;
        val
    }

    #[inline]
    fn write(&mut self, t: T) {
        if self.write_idx >= N {
            drop(unsafe { mem::replace(&mut self.arr[self.write_idx % N], MaybeUninit::new(t)).assume_init() });
        } else {
            self.arr[self.write_idx % N] = MaybeUninit::new(t);
        }
        self.write_idx += 1;
    }

    pub fn push(&mut self, t: T) -> Result<(), ()> {
        if self.write_idx < self.read_idx + N {
            self.write(t);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn pop(&mut self) -> Result<T, ()> {
        if self.read_idx < self.write_idx {
            Ok(self.read())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::{Rc, Weak};

    #[test]
    fn test_size_0() {
        let mut c = Circus::<u16, 0>::new();
        assert_eq!(c.pop(), Err(()));
        assert_eq!(c.push(1), Err(()));
    }

    #[test]
    fn test_size_1() {
        let mut c = Circus::<u16, 1>::new();
        for _ in 0..2 {
            assert_eq!(c.pop(), Err(()));
            assert_eq!(c.push(1), Ok(()));
            assert_eq!(c.push(2), Err(()));
            assert_eq!(c.pop(), Ok(1));
        }
    }

    #[test]
    fn test_size_2() {
        let mut c = Circus::<u16, 2>::new();
        for _ in 0..2 {
            assert_eq!(c.pop(), Err(()));
            assert_eq!(c.push(1), Ok(()));
            assert_eq!(c.push(2), Ok(()));
            assert_eq!(c.push(3), Err(()));
            assert_eq!(c.pop(), Ok(1));
            assert_eq!(c.push(4), Ok(()));
            assert_eq!(c.pop(), Ok(2));
            assert_eq!(c.pop(), Ok(4));
        }
    }

    #[test]
    fn test_ownership_management() {
        let mut c = Circus::<_, 1>::new();
        let val = 12345;
        let rc = Rc::new(val);
        let weak_rc = Rc::downgrade(&rc);
        assert_eq!(Weak::strong_count(&weak_rc), 1);
        c.push(rc).unwrap();
        assert_eq!(Weak::strong_count(&weak_rc), 1);
        let rc2 = c.pop().unwrap();
        assert_eq!(Rc::strong_count(&rc2), 1);
        assert_eq!(*rc2, 12345);
    }
}