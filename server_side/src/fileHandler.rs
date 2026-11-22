use std::fs::File;
static B : usize = 5 ; 
pub struct Header {
    pub root : i32 , 
    pub nbRec : i32  
}
pub struct  FileT{
    pub fileptr : File , 
    pub header : Header 
}
pub struct Block{
    pub values : [i32 ; B ], 
    pub childes :  [i32 ; B +1 ]
}
impl FileT {
    fn open(name : &str ){
        
    }
}