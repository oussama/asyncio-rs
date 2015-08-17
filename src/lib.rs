#![feature(box_raw)]


extern crate winapi;
extern crate ws2_32;
extern crate kernel32;
extern crate iocp;
extern crate libc;

use winapi::winsock2::*;
use winapi::ws2def::*;
use winapi::minwinbase::*;

use std::io::prelude::*;
use std::net::TcpStream;

use ws2_32::*;


use std::boxed;


use std::os::raw::{c_ulong,c_void};
use std::ptr::null;


use std::thread;
use std::boxed::Box;
use std::slice;

use std::collections::HashMap;

use std::os::windows::io::AsRawSocket;
use std::sync::{Mutex,Arc};

use std::cell::RefCell;
use std::borrow::Borrow;
use std::ops::Deref;
use std::str;

type CallbackStore<T> = Arc<Mutex<HashMap<usize, ( Box<Fn(T) + Send> , usize)  >>>;


struct AsyncIO {
    iocp:iocp::IoCompletionPort,
    handlers:CallbackStore<&'static [u8]>,
}

impl AsyncIO {

    
    pub fn new()->Option<Arc<Box<AsyncIO>>> {
        match iocp::IoCompletionPort::new(4)  {
            Ok(iocp) => {
                let mut aio_bare =AsyncIO {
                    iocp:iocp,
                    handlers:Arc::new(Mutex::new(HashMap::new())),
                };
                let mut aio_boxed = Box::new(aio_bare );
                let mut aio = Arc::new(aio_boxed);
                AsyncIO::init(aio.clone());
                return Some(aio);
            },
            Err(err) => {
                print!("Failed to start iocp");
                return None;
            } 
        }
    }
    
    pub fn register(&self,tcp_stream:&TcpStream){
        unsafe { self.iocp.associate(tcp_stream.as_raw_socket() as *mut c_void,0) };
    }

    
    
    fn init(aio:Arc<Box<AsyncIO>>){
    

        thread::spawn(move||{
            println!("aio thread started");
            loop {
            
                match aio.iocp.get_queued(600000) {
                   
                    Ok(status) => {
                        println!("Result {}",status.overlapped as usize);
                        let overlapped_ptr = status.overlapped as usize;
                        let handlers = aio.handlers.try_lock().unwrap();
                        let handler = handlers.get(&overlapped_ptr).unwrap();
                        
                        let wsa_buf_boxed = unsafe { Box::<WSABUF>::from_raw(handler.1 as *mut winapi::ws2def::WSABUF) };
                        let mut wsa_buf = *wsa_buf_boxed;
                        
                        // just to free it
                        let overlapped = unsafe { Box::<OVERLAPPED>::from_raw(status.overlapped) };
                        
                        let mut buf= unsafe { slice::from_raw_parts(wsa_buf.buf as *mut u8,wsa_buf.len as usize) };
                        handler.0( buf);
                    
                    },
                    Err(err) => {
                    
                        println!("get_queued error {}",err);
                    
                    }

                }
                
            }
        
        });
    
    
    }



}


trait AsyncRead  {
    fn async_read<F: Fn(&[u8]) + Send +'static >(&self,aio:&AsyncIO,handler:F);
}

impl AsyncRead  for TcpStream {

    fn async_read<F: Fn(&[u8]) + Send +'static >(&self,aio:&AsyncIO,handler:F){
        
        const BUFFER_SIZE:usize = 4096;
    
        let mut buf = [0;BUFFER_SIZE];
        let wsa_buf:Box<WSABUF> = Box::new( WSABUF{len:BUFFER_SIZE as u32,buf:buf.as_mut_ptr() as *mut i8 } );
        let lp_wsa_buf: *mut WSABUF = unsafe { Box::into_raw(wsa_buf) };
        
        let dw_buf_count = 1;
        let bytes_len: *mut u32 = unsafe { Box::into_raw(Box::new(0)) };
        
        let mut flags = 0;
        let lp_flags:*mut u32 = &mut flags;
        
        let overlapped = Box::new(OVERLAPPED {
            Internal: 0,
            InternalHigh: 0,
            Offset: 0,
            OffsetHigh: 0,
            hEvent: 0 as *mut c_void,
        });
        let lp_overlapped: *mut OVERLAPPED = unsafe { Box::into_raw(overlapped) };
        
        let buf_ptr = lp_wsa_buf as usize;
        
        let null : *mut libc::c_void;
        let err;
        unsafe {
            err = ws2_32::WSARecv(self.as_raw_socket(),lp_wsa_buf, dw_buf_count, bytes_len ,lp_flags, lp_overlapped, None);
        }
        aio.handlers.try_lock().unwrap().insert(lp_overlapped as usize,(Box::new(handler),buf_ptr));
    }


}




#[test]
fn simple_example() {

	let mut stream = TcpStream::connect("127.0.0.1:8124").unwrap();

	let mut aio = AsyncIO::new().unwrap();
	aio.register(&stream);
	stream.async_read(aio.deref(),|buf:&[u8]|{
		
		let s = match str::from_utf8(buf) {
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