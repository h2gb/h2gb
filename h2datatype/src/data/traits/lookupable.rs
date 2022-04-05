use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait Lookupable : Sized {
    type LookupBy : DeserializeOwned + Serialize;
    type LookupResult : DeserializeOwned + Serialize;
    type LookupOptions : Default;

    fn lookup_options(&self, value: &Self::LookupBy, options: Self::LookupOptions) -> Self::LookupResult;

    fn lookup(&self, value: &Self::LookupBy) -> Self::LookupResult {
        self.lookup_options(value, Default::default())
    }
}
