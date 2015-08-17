# asyncio-rs
Async IO for Rust

Goal: Extend the standar library net module to provide async io with callback hell

State
    currently only for windows
    TcpStream.read

Example:

    let mut stream = TcpStream::connect("127.0.0.1:8124").unwrap();
	let mut aio = AsyncIO::new().unwrap();
	aio.register(&stream);
	stream.async_read(aio.deref(),|buf:&[u8]|{
	
		let text = match str::from_utf8(buf).unwrap();
		println!("Text received: {}",text);
		
	});