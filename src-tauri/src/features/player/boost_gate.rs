use std::sync::{Mutex, MutexGuard};

static BOOST_GATE: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub(crate) static BOOST_TEST_GATE: Mutex<()> = Mutex::new(());

pub(crate) fn acquire_boost_gate() -> Result<MutexGuard<'static, ()>, String> {
    BOOST_GATE.try_lock().map_err(|_| {
        "a player or staff boost is already in progress; wait for it to finish".to_string()
    })
}
