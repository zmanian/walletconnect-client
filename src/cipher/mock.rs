use crate::cipher::RandProvider;
use chacha20poly1305::aead::{
    rand_core,
    rand_core::{CryptoRng, RngCore},
};
use rand::rngs::StdRng;

impl RandProvider for StdRng {}

#[derive(Clone)]
pub struct MockRandProvider {
    pub next_u32_call: u32,
    pub fill_bytes_call: u32,
}

impl RandProvider for MockRandProvider {}

impl RngCore for MockRandProvider {
    fn next_u32(&mut self) -> u32 {
        self.next_u32_call += 1;
        13
    }

    fn next_u64(&mut self) -> u64 {
        13
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.fill_bytes_call += 1;
        for i in 0..dest.len() {
            dest[i] = 8;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        for i in 0..dest.len() {
            dest[i] = 8;
        }
        Ok(())
    }
}

impl CryptoRng for MockRandProvider {}
