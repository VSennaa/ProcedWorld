//! Stable identifiers and deterministic allocators.

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        #[repr(transparent)]
        pub struct $name(pub u32);

        impl $name {
            pub const fn new(value: u32) -> Self { Self(value) }
            pub const fn value(self) -> u32 { self.0 }
        }
    };
}

define_id!(CivId);
define_id!(CityId);
define_id!(UnitId);
define_id!(TileIndex);
define_id!(TurnNumber);

impl TurnNumber {
    pub const ZERO: Self = Self(0);

    pub fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdAllocationError {
    Exhausted,
}

/// Separate monotonically increasing sequences ensure allocation does not depend
/// on interleaving unrelated entity types.
#[derive(Clone, Debug)]
pub struct IdAllocator {
    next_civ: Option<u32>,
    next_city: Option<u32>,
    next_unit: Option<u32>,
}

impl Default for IdAllocator {
    fn default() -> Self {
        Self { next_civ: Some(0), next_city: Some(0), next_unit: Some(0) }
    }
}

impl IdAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate_civ(&mut self) -> Result<CivId, IdAllocationError> {
        take_next(&mut self.next_civ).map(CivId)
    }

    pub fn allocate_city(&mut self) -> Result<CityId, IdAllocationError> {
        take_next(&mut self.next_city).map(CityId)
    }

    pub fn allocate_unit(&mut self) -> Result<UnitId, IdAllocationError> {
        take_next(&mut self.next_unit).map(UnitId)
    }
}

fn take_next(next: &mut Option<u32>) -> Result<u32, IdAllocationError> {
    let allocated = (*next).ok_or(IdAllocationError::Exhausted)?;
    *next = allocated.checked_add(1);
    Ok(allocated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_is_stable_per_entity_kind() {
        let mut ids = IdAllocator::new();
        assert_eq!(ids.allocate_civ(), Ok(CivId(0)));
        assert_eq!(ids.allocate_unit(), Ok(UnitId(0)));
        assert_eq!(ids.allocate_civ(), Ok(CivId(1)));
    }

    #[test]
    fn turns_start_at_zero() {
        assert_eq!(TurnNumber::ZERO, TurnNumber(0));
        assert_eq!(TurnNumber::ZERO.checked_next(), Some(TurnNumber(1)));
    }
}
