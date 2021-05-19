use simple_error::SimpleResult;

pub trait SizedOptions {
    fn to_s_i8(&self, v:   i8)   -> SimpleResult<String>;
    fn to_s_i16(&self, v:  i16)  -> SimpleResult<String>;
    fn to_s_i32(&self, v:  i32)  -> SimpleResult<String>;
    fn to_s_i64(&self, v:  i64)  -> SimpleResult<String>;
    fn to_s_i128(&self, v: i128) -> SimpleResult<String>;

    fn to_s_u8(&self, v:   u8)   -> SimpleResult<String>;
    fn to_s_u16(&self, v:  u16)  -> SimpleResult<String>;
    fn to_s_u32(&self, v:  u32)  -> SimpleResult<String>;
    fn to_s_u64(&self, v:  u64)  -> SimpleResult<String>;
    fn to_s_u128(&self, v: u128) -> SimpleResult<String>;

    fn to_s_f32(&self, v:  f32) -> SimpleResult<String>;
    fn to_s_f64(&self, v:  f64) -> SimpleResult<String>;
}
