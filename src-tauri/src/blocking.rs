/// Runs `work` on Tauri's blocking thread pool and returns its value.
///
/// A panic inside `work` resumes on the awaiting task, so a command keeps the
/// error type it had when its body ran inline.
pub async fn run<T, F>(work: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(work).await {
        Ok(value) => value,
        Err(tauri::Error::JoinError(error)) if error.is_panic() => {
            std::panic::resume_unwind(error.into_panic())
        }
        Err(error) => panic!("blocking work did not complete: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_returns_the_pool_thread_value_and_resumes_its_panic() {
        let caller = std::thread::current().id();
        let worker = tauri::async_runtime::block_on(run(|| std::thread::current().id()));
        assert_ne!(worker, caller);

        let payload = std::panic::catch_unwind(|| {
            tauri::async_runtime::block_on(run(|| -> () { panic!("blocking work failed") }))
        })
        .expect_err("panic reaches the awaiting caller");

        assert_eq!(
            payload.downcast_ref::<&str>(),
            Some(&"blocking work failed")
        );
    }
}
