use std::sync::{Mutex, MutexGuard};

static BOOST_GATE: Mutex<()> = Mutex::new(());
static LOAD_GATE: Mutex<()> = Mutex::new(());
static CONTEXT_GATE: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub(crate) static BOOST_TEST_GATE: Mutex<()> = Mutex::new(());

#[derive(Debug)]
pub(crate) struct BoostGuard {
    _boost: MutexGuard<'static, ()>,
    _load: MutexGuard<'static, ()>,
    _context: MutexGuard<'static, ()>,
}

pub(crate) struct LoadGuard {
    _load: MutexGuard<'static, ()>,
}

#[derive(Debug)]
pub(crate) struct ContextGuard {
    _context: MutexGuard<'static, ()>,
}

pub(crate) fn acquire_boost_gate() -> Result<BoostGuard, String> {
    let boost = BOOST_GATE.try_lock().map_err(|_| {
        "a player or staff boost is already in progress; wait for it to finish".to_string()
    })?;
    let load = match LOAD_GATE.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            return Err(
                "a Load Data operation is already in progress; wait for it to finish".to_string(),
            )
        }
    };
    let context = match CONTEXT_GATE.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            return Err("a save switch is already in progress; wait for it to finish".to_string())
        }
    };
    Ok(BoostGuard {
        _boost: boost,
        _load: load,
        _context: context,
    })
}

pub(crate) fn acquire_load_gate() -> Result<LoadGuard, String> {
    let load = LOAD_GATE.try_lock().map_err(|_| {
        // If BOOST is held, the Load Data contention is due to an active boost.
        if BOOST_GATE.try_lock().is_err() {
            "a player or staff boost is already in progress; wait for it to finish".to_string()
        } else {
            "a Load Data operation is already in progress; wait for it to finish".to_string()
        }
    })?;
    Ok(LoadGuard { _load: load })
}

pub(crate) fn acquire_context_gate() -> Result<ContextGuard, String> {
    let context = CONTEXT_GATE.try_lock().map_err(|_| {
        // CONTEXT contention is due to an active boost (which holds CONTEXT) or another switch.
        if BOOST_GATE.try_lock().is_err() {
            "a player or staff boost is already in progress; wait for it to finish".to_string()
        } else {
            "a save switch is already in progress; wait for it to finish".to_string()
        }
    })?;
    Ok(ContextGuard { _context: context })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_context_may_coexist() {
        let _guard = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let _load = acquire_load_gate().expect("acquire load");
        let context = acquire_context_gate();
        assert!(
            context.is_ok(),
            "context switch must succeed while load held"
        );
    }

    #[test]
    fn boost_fails_while_load_held() {
        let _guard = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let _load = acquire_load_gate().expect("acquire load");
        let boost = acquire_boost_gate();
        assert!(boost.is_err(), "boost must fail while load held");
        assert!(boost.unwrap_err().contains("Load Data"));
    }

    #[test]
    fn boost_fails_while_context_held() {
        let _guard = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let _ctx = acquire_context_gate().expect("acquire context");
        let boost = acquire_boost_gate();
        assert!(boost.is_err(), "boost must fail while context held");
    }

    #[test]
    fn context_fails_while_boost_held() {
        let _guard = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let _boost = acquire_boost_gate().expect("acquire boost");
        let ctx = acquire_context_gate();
        assert!(ctx.is_err(), "context switch must fail while boost held");
        assert!(ctx.unwrap_err().contains("boost"));
    }

    #[test]
    fn load_fails_while_boost_held() {
        let _guard = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let _boost = acquire_boost_gate().expect("acquire boost");
        let load = acquire_load_gate();
        assert!(load.is_err(), "load must fail while boost held");
    }

    #[test]
    fn load_fails_while_another_load_held() {
        let _guard = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let _load = acquire_load_gate().expect("acquire load");
        let second = acquire_load_gate();
        assert!(second.is_err(), "second load must fail");
    }
}
