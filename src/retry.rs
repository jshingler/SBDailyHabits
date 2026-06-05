use std::thread::sleep;
use std::time::Duration;
use tracing::warn;
use crate::error::HabitsError;

// Retries `f` up to 3 times on transient network errors (timeout, connection
// refused). Non-retryable errors (auth failures, bad JSON, missing data) are
// returned immediately. Uses a fixed backoff schedule: 2s → 5s → 10s.
pub fn with_retry<T, F>(label: &str, mut f: F) -> Result<T, HabitsError>
where
    F: FnMut() -> Result<T, HabitsError>,
{
    let delays = [2u64, 5, 10];

    for (attempt, &delay) in delays.iter().enumerate() {
        match f() {
            Ok(val) => return Ok(val),
            Err(e) if is_retryable(&e) => {
                warn!(
                    attempt = attempt + 1,
                    next_retry_secs = delay,
                    error = %e,
                    "{label} failed, retrying"
                );
                sleep(Duration::from_secs(delay));
            }
            Err(e) => return Err(e),
        }
    }

    f()
}

fn is_retryable(e: &HabitsError) -> bool {
    match e {
        HabitsError::Http(re) => re.is_timeout() || re.is_connect(),
        _ => false,
    }
}
