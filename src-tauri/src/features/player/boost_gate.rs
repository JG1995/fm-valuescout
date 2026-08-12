use std::sync::{Mutex, MutexGuard};

static PLAYER_BOOST_GATE: Mutex<()> = Mutex::new(());

pub(crate) fn acquire_player_boost_gate() -> Result<MutexGuard<'static, ()>, String> {
    PLAYER_BOOST_GATE
        .try_lock()
        .map_err(|_| "a player boost is already in progress; wait for it to finish".to_string())
}
