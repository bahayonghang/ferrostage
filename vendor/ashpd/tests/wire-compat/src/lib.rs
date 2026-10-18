#[cfg(test)]
#[path = "../../../src/flags.rs"]
mod flags;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use enumflags2::{BitFlags, bitflags};
    use serde::{Deserialize, Serialize};
    use zbus::{
        OwnedVariant, Type, Variant,
        wire::{
            LE,
            serialized::{Context, Data},
            to_bytes,
        },
    };

    #[bitflags]
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Capability {
        Pointer = 1,
        Keyboard = 2,
    }

    #[derive(Debug, Serialize, Deserialize, Type, PartialEq)]
    #[serde(transparent)]
    #[zbus(signature = "v")]
    struct Flags(#[serde(with = "super::flags")] BitFlags<Capability>);

    #[derive(Debug, Serialize, Deserialize, Type, PartialEq)]
    #[zbus(signature = "dict")]
    struct Options {
        #[serde(
            default,
            with = "super::flags::optional",
            skip_serializing_if = "Option::is_none"
        )]
        capabilities: Option<BitFlags<Capability>>,
    }

    #[test]
    fn flags_keep_unsigned_32_bit_variant_bytes() {
        let flags = Flags(Capability::Pointer | Capability::Keyboard);
        let encoded = to_bytes(Context::new(LE, 0), &flags).unwrap();
        assert_eq!(encoded.bytes(), &[1, b'u', 0, 0, 3, 0, 0, 0]);
        assert_eq!(encoded.deserialize::<Flags>().unwrap().0, flags);
    }

    #[test]
    fn unknown_flag_bits_and_wrong_wire_type_are_rejected() {
        for variant in [Variant::U32(4), Variant::U32(u32::MAX), Variant::I32(1)] {
            let encoded = to_bytes(Context::new(LE, 0), &variant).unwrap();
            assert!(encoded.deserialize::<Flags>().is_err());
        }
        let empty = Data::new(&[1, b'u', 0, 0, 0, 0, 0, 0][..], Context::new(LE, 0));
        assert!(empty.deserialize::<Flags>().unwrap().0.0.is_empty());
    }

    #[test]
    fn optional_flags_preserve_dictionary_omission_and_variant_type() {
        for capabilities in [None, Some(Capability::Keyboard.into())] {
            let options = Options { capabilities };
            let encoded = to_bytes(Context::new(LE, 0), &options).unwrap();
            assert_eq!(encoded.deserialize::<Options>().unwrap().0, options);
            let values = encoded
                .deserialize::<HashMap<String, OwnedVariant>>()
                .unwrap()
                .0;
            match capabilities {
                None => assert!(values.is_empty()),
                Some(_) => assert_eq!(
                    u32::try_from(values["capabilities"].try_clone().unwrap()).unwrap(),
                    2
                ),
            }
        }
        let bad = HashMap::from([("capabilities", Variant::U32(4))]);
        let encoded = to_bytes(Context::new(LE, 0), &bad).unwrap();
        assert!(encoded.deserialize::<Options>().is_err());
    }
}
