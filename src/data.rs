#[repr(C)]
pub struct Iovec {
    pub iov_base: u64,
    pub iov_len:u64,
}
#[derive(Debug,Clone)]
pub struct File {
}


