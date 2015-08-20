

#[cfg(windows)]
pub mod win;

#[cfg(windows)]
pub use self::win::{
    AsyncIO,
	AsyncRead,
};


#[cfg(unix)]
pub mod unix;

#[cfg(unix)]
pub use self::unix::{
    AsyncIO,
	AsyncRead,
};