use simple_error::SimpleResult;

pub trait TransformerTrait {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>>;

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>>;

    /// Check if the transformation will work.
    ///
    /// By default, we use a naive implementation that'll work in most
    /// circumstances. But if you have a more efficient way to check whether
    /// it'll successfully transform, I suggest doing that here.
    fn check(&self, buffer: &Vec<u8>) -> bool {
        self.transform(buffer).is_ok()
    }

    fn is_two_way(&self) -> bool;
}
