#[inline(always)]
pub fn c_string<T>(s: T) -> std::ffi::CString
where
    T: Into<Vec<u8>>,
{
    std::ffi::CString::new(s).unwrap()
}