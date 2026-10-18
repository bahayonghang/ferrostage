//! Preserve portal enumflags2 values as their specified unsigned D-Bus integer.

use enumflags2::{BitFlag, BitFlags};
use serde::{Deserializer, Serializer, de::Error as _};
use zbus::wire::as_variant;

pub(crate) fn serialize<F, S>(flags: &BitFlags<F>, serializer: S) -> Result<S::Ok, S::Error>
where
    F: BitFlag<Numeric = u32> + std::fmt::Debug,
    S: Serializer,
{
    as_variant::serialize(&flags.bits(), serializer)
}

pub(crate) fn deserialize<'de, F, D>(deserializer: D) -> Result<BitFlags<F>, D::Error>
where
    F: BitFlag<Numeric = u32> + std::fmt::Debug,
    D: Deserializer<'de>,
{
    let bits = as_variant::deserialize(deserializer)?;
    BitFlags::from_bits(bits).map_err(D::Error::custom)
}

pub(crate) mod optional {
    use super::*;

    pub(crate) fn serialize<F, S>(
        flags: &Option<BitFlags<F>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        F: BitFlag<Numeric = u32> + std::fmt::Debug,
        S: Serializer,
    {
        as_variant::optional::serialize(&flags.map(BitFlags::bits), serializer)
    }

    pub(crate) fn deserialize<'de, F, D>(deserializer: D) -> Result<Option<BitFlags<F>>, D::Error>
    where
        F: BitFlag<Numeric = u32> + std::fmt::Debug,
        D: Deserializer<'de>,
    {
        let bits: Option<u32> = as_variant::optional::deserialize(deserializer)?;
        bits.map(BitFlags::from_bits)
            .transpose()
            .map_err(D::Error::custom)
    }
}
