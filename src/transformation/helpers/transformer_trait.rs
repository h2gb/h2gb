use simple_error::SimpleResult;
use crate::transformation::Transformation;

pub trait TransformerTrait {
    /// A transform takes a buffer that's encoded and decodes it.
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>>;

    /// An untransform takes a buffer that's been decoded and re-encodes it
    /// (if possible).
    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>>;

    /// Check if the transformation will work.
    ///
    /// By default, we use a naive implementation that'll work in most
    /// circumstances. But if you have a more efficient way to check whether
    /// it'll successfully transform, I suggest doing that here.
    fn check(&self, buffer: &Vec<u8>) -> bool {
        self.transform(buffer).is_ok()
    }

    /// Can the transform be untransformed reliably?
    ///
    /// Importantly, if this true, then transform->untransform will return data
    /// that's the same length as the original, but not necessarily the same
    /// exact content.
    fn is_two_way(&self) -> bool;

    fn detect(buffer: &Vec<u8>) -> Vec<Transformation> where Self: Sized;
}
