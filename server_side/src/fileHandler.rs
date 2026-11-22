use std::fs::{File , OpenOptions };
use std::io::{self , Read , Write , Seek , SeekFrom };

static B : usize = 5 ; 
static HeaderSize : usize = 8 ; 
static BlockSize : usize = 8 * B + 4 ; 
pub struct Header {
    pub root : i32 , 
    pub nbRec : i32  , 
    pub lastBlock  : i32 
}
pub struct  FileT{
    pub fileptr : File , 
    pub header : Header 

}
pub struct Block{
    pub values : [i32 ; B ], 
    pub childs :  [i32 ; B +1 ]
}
impl FileT {
    fn open(name : &str , mode : char ) -> Result<FileT , io::Error > {
        let mut f = if mode == 'N' {
           OpenOptions::new().read(true).write(true).create(true).open(name)
        }else{
            OpenOptions::new().read(true).write(true).create(false).open(name)
        };
        let mut file = match f {
            Ok(f) => f , 
            Err(err) => return Err(err),
        };
        let mut header = if mode == 'N' {
            let mut  h = Header{
                 root : 0 , 
                 nbRec : 0 , 
                 lastBlock  : 0 
            };
            file.write_all(&h.root.to_le_bytes()).unwrap(); 
            file.write_all(&h.nbRec.to_le_bytes()).unwrap(); 
            h
        }else{
            
            let mut  rootBin = [0u8 ; 4];
            let mut nbRecBin = [0u8 ; 4];
            let mut lastBlockBin = [0u8 ; 4];
            file.read_exact(&mut rootBin).unwrap();
            file.read_exact(&mut nbRecBin).unwrap();
            file.read_exact(&mut lastBlockBin).unwrap();

            let mut h = Header {
                 root : i32::from_le_bytes(rootBin) , 
                 nbRec : i32::from_le_bytes(nbRecBin),
                 lastBlock : i32::from_le_bytes(lastBlockBin)

            };
            h
        };
        Ok(FileT{
            fileptr : file , header 
        })

    }
    fn read_block(file : &mut FileT , nb : usize )->Result<Block , io::Error> {
        file.fileptr.seek(SeekFrom::Start(HeaderSize as u64))? ; // to move pass the header 
        let offset = i64::try_from(nb * BlockSize)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Offset too large"
            ))?;

        file.fileptr.seek(SeekFrom::Current(offset))?;

        let mut values = [0i32 ; B]; 
        for value in values.iter_mut() {
            let mut buf = [0u8 ; 4] ;
            file.fileptr.read_exact(&mut buf); 
            *value = i32::from_le_bytes(buf); 
        }
        let mut childs = [0i32 ; B + 1 ]; 
        for value in values.iter_mut() {
            let mut buf = [0u8 ; 4] ;
            file.fileptr.read_exact(&mut buf); 
            *value = i32::from_le_bytes(buf); 
        }
        Ok(Block{
            values , childs 
        })
    }
    fn write_block(file : &mut FileT , block : & Block ) -> Result<i32 , io::Error> {
        file.fileptr.seek(SeekFrom::Start(HeaderSize as u64))? ; // to move pass the header 
        file.fileptr.seek(SeekFrom::End(0))?; 
        for value in block.values.iter() {
            file.fileptr.write_all(&value.to_le_bytes())? ; 
        }
         for value in block.childs.iter() {
            file.fileptr.write_all(&value.to_le_bytes())? ; 
        }
        file.header.lastBlock += 1 ; 
        
        Ok(1)
    }
}