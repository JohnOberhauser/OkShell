use std::cell::RefCell;
use gtk4_session_lock as session_lock;

thread_local! {
    static SESSION_LOCK: RefCell<Option<session_lock::Instance>> = RefCell::new(None);
}

pub fn session_lock() -> session_lock::Instance {
    SESSION_LOCK.with(|lock| {
        let mut lock = lock.borrow_mut();
        lock.get_or_insert_with(|| session_lock::Instance::new()).clone()
    })
}