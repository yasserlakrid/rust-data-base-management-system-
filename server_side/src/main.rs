use std::io::{Read, Write };
use std::net::{TcpListener , TcpStream};
use common::threadpool::ThreadPool ; 
use std::sync::{Arc , Mutex , atomic::{AtomicUsize , Ordering} };
mod startupDecoder ; 
mod  fileHandler ; 
use startupDecoder::split_startup; 
use fileHandler::{FileT };

static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);
    fn istablish_new_file(buf: &[u8] ){
        ACTIVE_CONNECTIONS.fetch_add(1 , Ordering::Relaxed ); 
        println!("the startup message is : {:?}" , String::from_utf8_lossy(buf)); 
        let startup_message =  split_startup(buf).unwrap() ; 
        println!("the startup message is : {} {} " , startup_message.user , startup_message.database);
        let mut new_file = FileT::open( &startup_message.database , 'N').unwrap(); 
       let test_keys: [i32; 50] = [
            472, 91, 835, 17, 263, 604, 958, 340, 725, 58,
            189, 511, 876, 402, 33, 697, 244, 780, 125, 569,
            913, 306, 648, 71, 487, 852, 199, 534, 761, 24,
            375, 920, 148, 683, 457, 806, 62, 291, 739, 518,
            884, 109, 366, 627, 975, 213, 590, 46, 798, 421,
        ];

        for key in test_keys {
            let root = new_file.header.root;
            new_file.insert(key, root).unwrap();
        }
            new_file.read(new_file.header.root);
    }

fn handleRequest(mut stream : TcpStream  ){   
    let mut buf = [1;1024]; 
    let n = stream.read(&mut buf).unwrap();
    let request = String::from_utf8_lossy(&buf[..n]); 
    if ACTIVE_CONNECTIONS.load(Ordering::Relaxed) == 0 {
        istablish_new_file(&buf[..n]); 
    }else{

    }
    
}
fn main() -> std::io::Result<()>{
    let cpt = Arc::new(AtomicUsize::new(0)) ; 

    let newPool = ThreadPool::new(4 ,  cpt);
    let listner = TcpListener::bind("127.0.0.1:5124").unwrap(); 
    println!("the server is listening at port 5124");
    for stream in listner.incoming() {
        let mut stream = stream?;
        newPool.sendtopool(|| {
            handleRequest(stream);
        }); 
    }
    println!("Hello, world!");
    Ok(())
}

