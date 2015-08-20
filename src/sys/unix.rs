extern crate winapi;
extern crate ws2_32;
extern crate kernel32;
extern crate iocp;
extern crate libc;
extern crate epoll;



use self::winapi::winsock2::*;
use self::winapi::ws2def::*;
use self::winapi::minwinbase::*;

use std::io::prelude::*;
use std::net::TcpStream;

use self::ws2_32::*;


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

use epoll;

type CallbackStore<T> = Arc<Mutex<HashMap<usize, ( Box<Fn(T) + Send> , usize)  >>>;


struct AsyncIO {
    epfd:i32,
    handlers:CallbackStore<&'static [u8]>,
}

impl AsyncIO {

    
    pub fn new()->Option<Arc<AsyncIO>> {
       /*
	   let epfd = 
       match epoll::create1(0) => {
            
            Ok(epfd) => {
               AsyncIO {
	   		         epfd:epfd,
                     handlers: Arc::new(Mutex::new(HashMap::new()));
	           }
            },
            
            Err(err) => {
                println!("Error crearing epoll {}",err);
                None
            }
       }
       
	   if epfd<0 {
        None;
	   }
       */
	   None
	   
    }
    
    pub fn register(&self,tcp_stream:&TcpStream){
    
		let event = epoll::epoll_event {
			events: epoll::EPOLLIN ,
			data: tcp_stream.as_raw_fd()
		};
        
		let err = epoll::epoll_ctl(self.epfd, epoll::EPOLLIN, tcp_stream.as_raw_fd() ,&event);
		// err should be 0 if success

    }

    
    
    fn init(aio:Arc<AsyncIO>){
	
		const MAXEVENTS: i32 = 10;
	
		let epfd = epoll_create1 (0);
        
		if epfd < 0 {
            println!("epoll_create1");
            return;
        }


        thread::spawn(move||{
        
            println!("aio thread started");
            
			let mut events = [epoll::epoll_event,MAXEVENTS];
            
            loop {

				let count = epoll::epoll_wait(epfd , &events,60);
				for i in 0..count {
				
					let fd = events[i].data;
					let stream = unsafe { TcpStream::from_raw_fd( &fd ) };
					
					let buf = stream.read();
					
					let handler = aio.handlers.get(fd).0;
					
					handler(buf);
				
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