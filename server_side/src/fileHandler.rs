use std::fs::{File , OpenOptions };
use std::io::{self , Read , Write , Seek , SeekFrom };

static B : usize = 5 ; 
static HeaderSize : usize = 8 ; 
static BlockSize : usize = 8 * B + 8 ; 
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
    pub degree : i32 , 
    pub values : [i32 ; B ], 
    pub childs :  [i32 ; B +1 ]
}
impl FileT {
    pub fn open(name : &str , mode : char ) -> Result<Self , io::Error > {
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
    fn read_block(&mut self , nb : usize )->Result<Block , io::Error> {
        self.fileptr.seek(SeekFrom::Start(HeaderSize as u64))? ; // to move pass the header 
        let offset = i64::try_from(nb * BlockSize)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Offset too large"
            ))?;

        self.fileptr.seek(SeekFrom::Current(offset))?;
       
        let degree = { 
            let mut buf = [0u8 ; 4] ;
            self.fileptr.read_exact(&mut buf); 
            i32::from_le_bytes(buf)
        };
        
        let mut values = [0i32 ; B]; 
        for value in values.iter_mut() {
            let mut buf = [0u8 ; 4] ;
            self.fileptr.read_exact(&mut buf); 
            *value = i32::from_le_bytes(buf); 
        }
        let mut childs = [0i32 ; B + 1 ]; 
        for value in values.iter_mut() {
            let mut buf = [0u8 ; 4] ;
            self.fileptr.read_exact(&mut buf); 
            *value = i32::from_le_bytes(buf); 
        }
        Ok(Block{
            degree , values , childs 
        })
    }
    fn write_block( &mut self , block : & Block , nb  : usize ) -> Result<i32 , io::Error> {
        self.fileptr.seek(SeekFrom::Start(HeaderSize as u64))? ; // to move pass the header 
         let offset = i64::try_from(nb * BlockSize)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Offset too large"
            ))?;

        self.fileptr.seek(SeekFrom::Current(offset))?;

        self.fileptr.write_all(&block.degree.to_le_bytes())?; 
        for value in block.values.iter() {
            self.fileptr.write_all(&value.to_le_bytes())? ; 
        }
         for value in block.childs.iter() {
            self.fileptr.write_all(&value.to_le_bytes())? ; 
        }
        self.header.lastBlock += 1 ; 

        Ok(1)
    }
}
#[derive(Debug, PartialEq)]
enum InsertError {
    Full,       // no free slot left (in a B-tree: time to split)
    Duplicate,  // key already present
}

/// Inserts `key` into the sorted prefix `values[..n]`.
/// Returns the index where it was inserted.
fn insert_in_array(values: &mut [i32], n: usize, key: i32) -> Result<usize, InsertError> {
    if n >= values.len() {
        return Err(InsertError::Full);
    }

    match values[..n].binary_search(&key) {
        Ok(_) => Err(InsertError::Duplicate),
        Err(pos) => {
            values.copy_within(pos..n, pos + 1); // shift right (like memmove)
            values[pos] = key;
            Ok(pos)
        }
    }
}

impl FileT {
   pub fn insert(&mut self , key : i32 ) -> Result<i32 , io::Error> {
        
        let mut  root = self.read_block( self.header.root as usize ).unwrap(); 
        if root.degree < B as i32 {
            insert_in_array(&mut root.values , B , key); 
            self.write_block(&root , self.header.root as usize  ); 
        } else {
            if key < root.values[0] {
                
                let mut leftSubTree = FileT {
                    header : Header {
                        root : root.childs[0] , 
                        nbRec : self.header.nbRec , //these doesn't matter in this operation 
                        lastBlock  : self.header.lastBlock // this too 
                    } , 
                    fileptr : self.fileptr.try_clone().unwrap()
                };
                leftSubTree.insert(key); 
            }else if  key > root.values[B-1]  {

                let mut rightSubTree = FileT {
                    header : Header {
                        root : root.childs[B] , 
                        nbRec : self.header.nbRec , //these doesn't matter in this operation 
                        lastBlock  : self.header.lastBlock // this too 
                    } , 
                    fileptr : self.fileptr.try_clone().unwrap()
                   
                };
                rightSubTree.insert(key); 
            }else {
                for i in 0..root.values.len() - 1 {
                    if root.values[i] < key && root.values[i + 1 ] > key  {
                        let mut middleSubTree = FileT {
                            header : Header {
                                root : root.childs[i] , 
                                nbRec : self.header.nbRec , //these doesn't matter in this operation 
                                lastBlock  : self.header.lastBlock // this too 
                            } , 
                    fileptr : self.fileptr.try_clone().unwrap()
                          
                        };
                        middleSubTree.insert(key); 
                    }
                }
            }
            
        }
        
        Ok(1)
   }
}