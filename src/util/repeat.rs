use std::cmp::Ordering;
use std::mem::{replace, MaybeUninit};

pub struct Repeat<T: Clone> {
    t: MaybeUninit<T>,
    n: usize,
}

impl<T: Clone> Iterator for Repeat<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.n.cmp(&1) {
            Ordering::Greater => {
                self.n -= 1;
                Some(unsafe { self.t.assume_init_ref() }.clone())
            }
            Ordering::Equal => {
                self.n -= 1;
                let original = replace(&mut self.t, MaybeUninit::uninit());
                Some(unsafe { original.assume_init() })
            }
            Ordering::Less => None,
        }
    }
}

pub fn repeat<T: Clone>(t: T, n: usize) -> Repeat<T> {
    Repeat {
        t: MaybeUninit::new(t),
        n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::cmp::max;
    use std::rc::Rc;

    #[derive(Eq, PartialEq)]
    struct CloneCounter {
        count: Rc<RefCell<usize>>,
    }

    impl CloneCounter {
        fn new() -> Self {
            Self {
                count: Rc::new(RefCell::new(0)),
            }
        }

        fn count(&self) -> usize {
            *(*self.count).borrow()
        }
    }

    impl Clone for CloneCounter {
        fn clone(&self) -> Self {
            *(self.count).borrow_mut() += 1;
            Self {
                count: Rc::clone(&(self.count)),
            }
        }
    }

    #[test]
    fn test_clone_counter() {
        let c1 = CloneCounter::new();
        assert_eq!(c1.count(), 0);
        let c2 = c1.clone();
        assert_eq!(c1.count(), 1);
        assert_eq!(c2.count(), 1);
        let c3 = c1.clone();
        assert_eq!(c1.count(), 2);
        assert_eq!(c2.count(), 2);
        assert_eq!(c3.count(), 2);
        let c4 = c2.clone();
        assert_eq!(c1.count(), 3);
        assert_eq!(c2.count(), 3);
        assert_eq!(c3.count(), 3);
        assert_eq!(c4.count(), 3);
    }

    #[test]
    fn test_repeat() {
        for n in 0..=3 {
            let c = CloneCounter::new();
            let my_clone = c.clone();
            let r = repeat(c, n).collect::<Vec<_>>();
            let expected_clones = max(1, n);
            assert_eq!(r.len(), n);
            assert_eq!(my_clone.count(), expected_clones);
            r.iter()
                .for_each(|c| assert_eq!(c.count(), expected_clones));
        }
    }
}
