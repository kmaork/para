use std::sync::{Arc, Condvar, Mutex};

pub(crate) enum Notification {
    // There might be new work. There might also be no work at all, because we might have been
    // spuriously woken up, or other threads have beaten us to it.
    NewWork,
    Done,
}

struct BlockingThreadTrack {
    blocking_threads: usize,
    total_threads: usize,
    is_done: bool,
}
struct NotifierInner {
    blocking_track: Mutex<BlockingThreadTrack>,
    condvar: Condvar,
}

impl NotifierInner {
    fn new() -> Self {
        Self {
            blocking_track: Mutex::new(BlockingThreadTrack {
                total_threads: 1,
                blocking_threads: 0,
                is_done: false,
            }),
            condvar: Condvar::new(),
        }
    }
}

pub(crate) struct Notifier {
    inner: Arc<NotifierInner>,
}

impl Notifier {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(NotifierInner::new()),
        }
    }

    pub(crate) fn wait_for_work(&self) -> Notification {
        let mut blocking_track = self.inner.blocking_track.lock().unwrap();
        debug_assert!(!blocking_track.is_done);
        blocking_track.blocking_threads += 1;
        if blocking_track.blocking_threads == blocking_track.total_threads {
            blocking_track.is_done = true;
            self.inner.condvar.notify_all();
            return Notification::Done;
        }
        blocking_track = self.inner.condvar.wait(blocking_track).unwrap();
        if blocking_track.is_done {
            Notification::Done
        } else {
            Notification::NewWork
        }
    }

    pub(crate) fn added_work(&self) {
        // TODO: don't really need to wake all of them up
        self.inner.condvar.notify_all();
    }
}

impl Clone for Notifier {
    fn clone(&self) -> Self {
        self.inner.blocking_track.lock().unwrap().total_threads += 1;
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Drop for Notifier {
    fn drop(&mut self) {
        self.inner.blocking_track.lock().unwrap().total_threads -= 1;
    }
}
