#[macro_export]
macro_rules! include_flash_bytes {
    ($file:expr) => {
        unsafe { include!(concat!(env!("OUT_DIR"), "/", $file)) }
    };
}

#[macro_export]
macro_rules! include_flash_str {
    ($file:expr) => {
        str::from_utf8(unsafe { include!(concat!(env!("OUT_DIR"), "/", $file)) }).unwrap()
    };
}
