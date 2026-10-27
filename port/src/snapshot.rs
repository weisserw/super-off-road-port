//! Array codecs keep fixed native capacities while serializing every element.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub mod array {
    use super::*;
    pub fn serialize<T: Serialize, S: Serializer, const N: usize>(
        value: &[T; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_slice().serialize(serializer)
    }
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<[T; N], D::Error> {
        Vec::<T>::deserialize(deserializer)?
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid fixed array capacity"))
    }
}
pub mod boxed_array {
    use super::*;
    pub fn serialize<T: Serialize, S: Serializer, const N: usize>(
        value: &[T; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_slice().serialize(serializer)
    }
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<Box<[T; N]>, D::Error> {
        Vec::<T>::deserialize(deserializer)?
            .into_boxed_slice()
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid boxed array capacity"))
    }
}
