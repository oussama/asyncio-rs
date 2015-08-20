#![feature(box_raw)]




mod sys;

#[cfg(unix)]
pub use sys::unix::{
    AsyncIO,
    AsyncRead,
};


#[cfg(windows)]
pub use sys::win::{
    AsyncIO,
    AsyncRead,
};




