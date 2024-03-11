
pub type Rand = fn(&mut [u8]);

pub fn dummy_rand(_buf: &mut [u8]) {}

#[cfg(feature = "std")]
pub fn sys_rand(buf: &mut [u8]) {
    use rand::{thread_rng, RngCore};

    thread_rng().fill_bytes(buf);
}

#[cfg(feature = "riot-os")]
use rand_core::{RngCore, Error, impls};
#[cfg(feature = "riot-os")]
struct CountingRng(u64);

#[cfg(feature = "riot-os")]
impl RngCore for CountingRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

#[cfg(feature = "riot-os")]
static mut RNG: CountingRng = CountingRng(123);

#[cfg(feature = "riot-os")]
pub fn riot_rand(buf: &mut [u8]) {
    // TODO: Fill buf with random bytes without std
    unsafe { RNG.fill_bytes(buf); }
}