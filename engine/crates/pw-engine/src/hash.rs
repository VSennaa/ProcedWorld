//! Stable hashing and canonical byte serialization for simulation state.

/// FNV-1a 64-bit offset basis.
pub const FNV1A_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
pub const FNV1A_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Computes the platform-independent FNV-1a 64-bit hash of `bytes`.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = FNV1A_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV1A_PRIME);
    }
    hash
}

/// Hash of a canonically serialized simulation state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateHash(pub u64);

impl StateHash {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Builds canonical state bytes in the caller-defined, fixed field order.
///
/// All multi-byte scalar methods use little-endian representation. Container
/// ordering is deliberately the caller's responsibility: simulation callers must
/// visit entities in stable ID order.
#[derive(Clone, Debug, Default)]
pub struct StateHasher {
    bytes: Vec<u8>,
}

impl StateHasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { bytes: Vec::with_capacity(capacity) }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn write_i8(&mut self, value: i8) {
        self.bytes.push(value as u8);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i16(&mut self, value: i16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn finish(&self) -> StateHash {
        StateHash(fnv1a(&self.bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_has_a_stable_empty_value() {
        assert_eq!(fnv1a(b""), FNV1A_OFFSET_BASIS);
    }

    #[test]
    fn state_hasher_uses_little_endian_fields() {
        let mut hasher = StateHasher::new();
        hasher.write_u16(0x1234);
        hasher.write_i32(-2);
        hasher.write_bool(true);
        assert_eq!(hasher.finish(), StateHash(fnv1a(&[0x34, 0x12, 0xfe, 0xff, 0xff, 0xff, 1])));
    }
}
