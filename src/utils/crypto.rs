use rand::RngCore;

/// Generates cryptographically random bytes.
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

/// Generates a hex-encoded random string of N bytes.
pub fn random_hex<const N: usize>() -> String {
    hex::encode(random_bytes::<N>())
}
