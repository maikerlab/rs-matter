use core::time::Duration;
use log::debug;

pub type Epoch = fn() -> Duration;

// As per the spec, if Not After is 0, it should set the time to GeneralizedTime value of
// 99991231235959Z
// So CERT_DOESNT_EXPIRE value is calculated as epoch(99991231235959Z) - MATTER_EPOCH_SECS
pub const MATTER_CERT_DOESNT_EXPIRE: u64 = 252455615999;

pub const MATTER_EPOCH_SECS: u64 = 946684800; // Seconds from 1970/01/01 00:00:00 till 2000/01/01 00:00:00 UTC

pub fn dummy_epoch() -> Duration {
    Duration::from_secs(0)
}

#[cfg(feature = "std")]
pub fn sys_epoch() -> Duration {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
}

#[cfg(feature = "riot-os")]
fn get_timestamp() -> u64 {
    static mut COUNTER: u64 = MATTER_EPOCH_SECS;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

#[cfg(feature = "riot-os")]
pub fn riot_epoch() -> Duration {
    Duration::from_secs(get_timestamp())
}