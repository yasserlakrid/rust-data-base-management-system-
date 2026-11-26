static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

use std::io::{Read, Write };
use std::net::{TcpListener , TcpStream};
use common::threadpool::ThreadPool ; 
use std::sync::{Arc , Mutex , atomic::{AtomicUsize , Ordering} };
mod requestDecoder ; 
mod  fileHandler ; 
use requestDecoder::{split_startup , split_request , RequestT , DecodeError }; 

use fileHandler::{FileT  , InsertError};

fn istablish_new_file(buf: &[u8] ){
    ACTIVE_CONNECTIONS.fetch_add(1 , Ordering::Relaxed ); 
    println!("creating a new  file");
   
    let startup_message =  split_startup(buf).unwrap() ; 
  
    let mut new_file = FileT::open( &startup_message.database , 'N').unwrap(); 
    let test_keys: [i32; 50] = [
        472, 91, 835, 15, 263, 604, 958, 340, 725, 58,
        189, 511, 876, 402, 33, 697, 244, 780, 125, 569,
        913, 306, 648, 71, 487, 852, 199, 534, 761, 24,
        375, 920, 148, 683, 457, 806, 62, 291, 739, 518,
        884, 109, 366, 627, 975, 213, 590, 46, 798, 421,
    ];  
    for key in test_keys {
        let root = new_file.header.root;
        new_file.insert(key, root).unwrap();
    }
    println!("the number of blocks in this file is {}" , new_file.header.lastBlock); 

        new_file.read(new_file.header.root);
}

fn performRequest(buf : &[u8] ,mut  stream : & TcpStream  )-> Result<i32 , DecodeError>{
    println!("oppening an existing file");
    let mut new_file = FileT::open( "data.bin", 'O').unwrap(); 


    let request = split_request(buf ,ACTIVE_CONNECTIONS.load(Ordering::Relaxed)  ); 
    let request = match request {
        Ok(request) => request, 
        Err(err) => return Err(err)
    }; 

    match request.typeR {
        RequestT::Insert => {
            println!("inserting the { } value into the file " ,request.body ); 
            match new_file.insert(request.body , new_file.header.root) {
                        
                Ok(some) => { stream.write_all("200 OK".as_bytes()) ;} ,
                Err(err) => {match err {
                    InsertError::Full => {
                        stream.write_all("104 Full".as_bytes()) ;
                    } ,
                    InsertError::Duplicate => {
                        stream.write_all("104 Duplicated".as_bytes()) ;
                    }
                } }
            }
          
        }, 
        RequestT::Read => {
            println!("reading the file" ); 
           
            stream.write_all( new_file.read(new_file.header.root).as_bytes()) ;
        },
        _=>{
            println!("request not availbe for now"); 
        }
    }
    Ok(1)
}
fn handleRequest(mut stream : TcpStream  ){   
  loop{
 let mut buf = [1;1024]; 
    let n = stream.read(&mut buf).unwrap();
    if n == 0 {break;}
    let request = String::from_utf8_lossy(&buf[..n]);
     
    println!("the request is {:?}" , request  ); 
    if let Some(c) = buf.get(4) {
        println!("the first character is {}" ,c  ); 
        if c == &b'S' {
            istablish_new_file(&buf[..n]); 
            stream.write_all("new file created".as_bytes()); 
        }else{
            performRequest(&buf[..n] , &stream); 
        };
    }

    }
   
    
  }

    

fn main() -> std::io::Result<()>{
    let cpt = Arc::new(AtomicUsize::new(0)) ; 

    let newPool = ThreadPool::new(4 ,  cpt);
    let listner = TcpListener::bind("127.0.0.1:5122").unwrap(); 
    println!("the server is listening at port 5124");
    for stream in listner.incoming() {
        println!("new connection comes "); 
        let mut stream = stream?;
        newPool.sendtopool(move || {
            handleRequest(stream);
        }); 
    }
    println!("Hello, world!");
    Ok(())
}

