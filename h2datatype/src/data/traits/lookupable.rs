use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait Lookupable : Sized {
    type LookupBy : DeserializeOwned + Serialize;
    type LookupResult : DeserializeOwned + Serialize;
    type LookupOptions;

    fn lookup(&self, value: &Self::LookupBy, options: Self::LookupOptions) -> Self::LookupResult;
}
