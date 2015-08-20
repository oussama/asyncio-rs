extern crate asyncio;

use std::net::TcpStream;
use asyncio::{AsyncIO,AsyncRead};

use std::str;

#[test]
fn simple_example() {

	let stream = TcpStream::connect("127.0.0.1:8124").unwrap();

	let aio = AsyncIO::new().unwrap();
	aio.register(&stream);
	stream.async_read(&aio,|buf:&[u8]|{
		
		match str::from_utf8(buf) {
			Ok(v) => {
				println!("result: {}", v);
				v
			},
			Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
		};

	});
	
	
	// this loop is just to prevent closing, too lazey to get a bette solution ;D
    loop {
    
    }
	
}