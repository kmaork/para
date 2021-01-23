use crossbeam::utils::CachePadded;
use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;
use std::{fmt, mem};

pub struct CantPush<T> {
    pub t: T,
}

impl<T> Debug for CantPush<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Can't push - queue is full")
    }
}

pub struct Circus<T, const N: usize> {
    arr: [MaybeUninit<T>; N],
    write_idx: CachePadded<usize>,
    read_idx: CachePadded<usize>,
}

impl<T, const N: usize> Circus<T, N> {
    pub(crate) fn new() -> Self {
        Circus {
            arr: MaybeUninit::uninit_array(),
            write_idx: CachePadded::new(0),
            read_idx: CachePadded::new(0),
        }
    }

    #[inline]
    fn read(&mut self) -> T {
        let val = unsafe {
            mem::replace(&mut self.arr[*self.read_idx % N], MaybeUninit::uninit()).assume_init()
        };
        *self.read_idx += 1;
        val
    }

    #[inline]
    pub fn can_push(&self) -> bool {
        *self.write_idx < *self.read_idx + N
    }

    #[inline]
    fn write(&mut self, t: T) {
        self.arr[*self.write_idx % N] = MaybeUninit::new(t);
        *self.write_idx += 1;
    }

    #[inline]
    pub fn push(&mut self, t: T) -> Result<(), CantPush<T>> {
        if self.can_push() {
            self.write(t);
            Ok(())
        } else {
            Err(CantPush { t })
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Result<T, ()> {
        if *self.read_idx < *self.write_idx {
            Ok(self.read())
        } else {
            Err(())
        }
    }
}

impl<T, const N: usize> Iterator for Circus<T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.pop().ok()
    }
}

impl<T, const N: usize> Drop for Circus<T, N> {
    fn drop(&mut self) {
        for item in self {
            drop(item)
        }
    }
}

// We use strings in the tests to better test the unsafe memory management.
// Strings, (as opposed to numbers for example) have non-trivial destructors.
#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::{Rc, Weak};

    #[test]
    fn test_size_0() {
        let mut c = Circus::<u16, 0>::new();
        assert_eq!(c.pop(), Err(()));
        assert_eq!(c.push(1).unwrap_err().t, 1);
    }

    #[test]
    fn test_size_1() {
        let mut c = Circus::<_, 1>::new();
        for _ in 0..2 {
            c.pop().unwrap_err();
            c.push(String::from("a")).unwrap();
            assert_eq!(c.push(String::from("b")).unwrap_err().t, "b");
            assert_eq!(c.pop().unwrap(), String::from("a"));
        }
    }

    #[test]
    fn test_size_2() {
        let mut c = Circus::<_, 2>::new();
        for _ in 0..2 {
            assert_eq!(c.pop(), Err(()));
            c.push(String::from("a")).unwrap();
            c.push(String::from("b")).unwrap();
            assert_eq!(c.push(String::from("c")).unwrap_err().t, "c");
            assert_eq!(c.pop().unwrap(), "a");
            c.push(String::from("d")).unwrap();
            assert_eq!(c.pop().unwrap(), "b");
            assert_eq!(c.pop().unwrap(), "d");
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

    #[test]
    fn test_dropping() {
        let mut c = Circus::<_, 1>::new();
        let rc = Rc::new(12345);
        let weak_rc = Rc::downgrade(&rc);
        assert_eq!(Weak::strong_count(&weak_rc), 1);
        c.push(rc).unwrap();
        assert_eq!(Weak::strong_count(&weak_rc), 1);
        drop(c);
        assert_eq!(Weak::strong_count(&weak_rc), 0);
    }
}
