use std::net::{TcpListener  , TcpStream}; 
use std::io::{Write , Read}; 
use std::thread ;

use common::threadpool::ThreadPool;
use common::dbconnection::DBconnection; 

//i will create a data base connection to the address of the server program 
//the server will create files and manages requests 

fn main() ->std::io::Result<()> {
    let dbconnection = DBconnection::new( "yasser" , "data.bin", "127.0.0.1:5122").unwrap();
    dbconnection.send("I 3 10");
    dbconnection.send("R");

    Ok(())
}
