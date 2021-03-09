use crossbeam::utils::CachePadded;
use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, mem};

pub struct CantPush<T>(pub T);

impl<T> Debug for CantPush<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Can't push - queue is full")
    }
}

struct CircusIndexes<const N: usize> {
    write_idx: CachePadded<AtomicUsize>,
    read_idx: CachePadded<AtomicUsize>,
}

impl<const N: usize> CircusIndexes<N> {
    pub(crate) fn new() -> Self {
        Self {
            write_idx: CachePadded::new(AtomicUsize::new(0)),
            read_idx: CachePadded::new(AtomicUsize::new(0)),
        }
    }

    #[inline]
    pub fn can_push(&self) -> bool {
        self.write_idx.load(Ordering::SeqCst) < self.read_idx.load(Ordering::SeqCst) + N
    }

    #[inline]
    pub fn can_pop(&self) -> bool {
        self.read_idx.load(Ordering::SeqCst) < self.write_idx.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn get_read_idx_and_increment(&self) -> usize {
        self.read_idx.fetch_add(1, Ordering::SeqCst) % N
    }

    #[inline]
    pub fn get_write_idx_and_increment(&self) -> usize {
        self.write_idx.fetch_add(1, Ordering::SeqCst) % N
    }
}

// Draw inspiration from https://github.com/tokio-rs/tokio/blob/master/tokio/src/runtime/queue.rs#L23
//    Intrusive linked list of tasks?
//    Use vec to allocate buffer instead on stack?
//
// What happens when another thread has a reference to this queue and this thread destroys the queue? Sounds like weakref
// When backpopping, other threads must read the read_idx (to see if it's smaller than write_idx) and decrement write_idx. That means they both must be atomic.
// So we need a struct with a std::sync::Weak pointing at our indexes, able to mutate us. How? Refcell? how is it done here https://github.com/tokio-rs/tokio/blob/master/tokio/src/runtime/queue.rs#L295
// Make the indexes into a struct of its own to manage all atomicity and with normal interface. Create a Stealer struct with a reference to that struct
pub struct Circus<T, const N: usize> {
    arr: [MaybeUninit<T>; N],
    indexes: CircusIndexes<N>,
}

impl<T, const N: usize> Circus<T, N> {
    pub(crate) fn new() -> Self {
        Circus {
            arr: MaybeUninit::uninit_array(),
            indexes: CircusIndexes::new(),
        }
    }

    #[inline]
    fn read(&mut self) -> T {
        let val = unsafe {
            mem::replace(
                &mut self.arr[self.indexes.get_read_idx_and_increment()],
                MaybeUninit::uninit(),
            )
            .assume_init()
        };
        val
    }

    #[inline]
    fn write(&mut self, t: T) {
        self.arr[self.indexes.get_write_idx_and_increment()] = MaybeUninit::new(t);
    }

    #[inline]
    pub fn push(&mut self, t: T) -> Result<(), CantPush<T>> {
        if self.indexes.can_push() {
            self.write(t);
            Ok(())
        } else {
            Err(CantPush(t))
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Result<T, ()> {
        if self.indexes.can_pop() {
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
        assert_eq!(c.push(1).unwrap_err().0, 1);
    }

    #[test]
    fn test_size_1() {
        let mut c = Circus::<_, 1>::new();
        for _ in 0..2 {
            c.pop().unwrap_err();
            c.push(String::from("a")).unwrap();
            assert_eq!(c.push(String::from("b")).unwrap_err().0, "b");
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
            assert_eq!(c.push(String::from("c")).unwrap_err().0, "c");
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
