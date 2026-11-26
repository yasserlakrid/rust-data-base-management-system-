use std::fs::{File , OpenOptions };
use std::io::{self , Read , Write , Seek , SeekFrom };

static B : usize = 5 ; 
static HeaderSize : usize = 12 ; 
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
     fn read_block(&mut self , nb : usize )->Result<Block , io::Error> {
        self.fileptr.seek(SeekFrom::Start(0)); 
        self.fileptr.seek(SeekFrom::Start(HeaderSize as u64))? ; // to move pass the header 
        let offset = i64::try_from(nb * BlockSize)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Offset too large"
            ))?;

        self.fileptr.seek(SeekFrom::Current(offset))?;
       
        let mut degree = { 
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
        for value in childs.iter_mut() {
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
       
        Ok(1)
    
    }

    pub fn open(name : &str , mode : char ) -> Result<Self , io::Error > {
        let mut f = if mode == 'N' {
           OpenOptions::new().read(true).write(true).create(true).open(name)
        }else{
            OpenOptions::new().read(true).write(true).create(false).open(name)
        };
        let mut newfile = match f {
            Ok(f) => f , 
            Err(err) => return Err(err),
        };
        let mut header = if mode == 'N' {
            let mut  h = Header{
                 root : 0 , 
                 nbRec : 0 , 
                 lastBlock  : 0 
            };
           
            newfile.write_all(&h.root.to_le_bytes()).unwrap(); 
            newfile.write_all(&h.nbRec.to_le_bytes()).unwrap(); 
            newfile.write_all(&h.lastBlock.to_le_bytes()).unwrap(); 

            

            h
        }else{
            
            let mut  rootBin = [0u8 ; 4];
            let mut nbRecBin = [0u8 ; 4];
            let mut lastBlockBin = [0u8 ; 4];
            newfile.read_exact(&mut rootBin).unwrap();
            newfile.read_exact(&mut nbRecBin).unwrap();
            newfile.read_exact(&mut lastBlockBin).unwrap();

            println!("the file openned has {} blocks " ,  i32::from_le_bytes(lastBlockBin)); 
            let mut h = Header {
                 root : i32::from_le_bytes(rootBin) , 
                 nbRec : i32::from_le_bytes(nbRecBin),
                 lastBlock : i32::from_le_bytes(lastBlockBin)

            };
            h
        };
        let mut filet = FileT{
                fileptr : newfile , header 
        };
          
        //initialize the first block 
        if mode == 'N' || mode == 'n' {
         let mut firstBlock = Block {
                    degree : 0 ,
                    values : [0i32 ; B],
                    childs : [0i32 ; B+1]
                };
            filet.write_block(&firstBlock , 0); 
        }
            


        Ok(filet)

    }
   
}
#[derive(Debug, PartialEq)]
pub enum InsertError {
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
    pub fn append_new_block(&mut self){
       
        let mut firstBlock = Block {
                degree : 0 ,
                values : [0i32 ; B],
                childs : [0i32 ; B+1]
            };

        self.write_block(&firstBlock , (self.header.lastBlock + 1) as usize  );
        self.header.lastBlock += 1 ; 

        self.fileptr.seek(SeekFrom::Start(0));
        self.fileptr.write_all(& self.header.root.to_le_bytes()).unwrap(); 
        self.fileptr.write_all(& self.header.nbRec.to_le_bytes()).unwrap(); 
        self.fileptr.write_all(& self.header.lastBlock.to_le_bytes()).unwrap(); 
            
    }
   pub fn insert(&mut self , key : i32 , root : i32 ) -> Result<i32 , InsertError > {

        let mut  rootBlock = self.read_block( root as usize ).unwrap(); 
        let mut i = 0 ; 
        if rootBlock.degree < B as i32 {
            match  insert_in_array(&mut rootBlock.values , rootBlock.degree as usize , key) {
                Ok(some) => { 
                    rootBlock.degree += 1 ; 
                    self.write_block(&rootBlock , root as usize  );  
                    return  Ok(1) 
                },

                Err(err) => {return Err(InsertError::Duplicate ); }
            }
           
           
           
          

        } else {
            if key < rootBlock.values[0] {
                if rootBlock.childs[0] == 0 {
                    self.append_new_block(); 
                    rootBlock.childs[0] = self.header.lastBlock ;
                    self.write_block(&rootBlock , root as usize );
                   
                }
                    return self.insert(key , rootBlock.childs[0] ); 
                

            }else if  key > rootBlock.values[B-1]  {
                 if rootBlock.childs[B] == 0 {
                    self.append_new_block(); 
                    rootBlock.childs[B] = self.header.lastBlock ;
                    self.write_block(&rootBlock , root as usize );
                }
                    return self.insert(key , rootBlock.childs[B] );

            }else {
                let mut ij = Ok(1) ; 
                for i in 0..rootBlock.values.len() - 1 {
                    if rootBlock.values[i] < key && rootBlock.values[i + 1 ] > key  {
                         if rootBlock.childs[i] == 0 {
                            self.append_new_block(); 
                            rootBlock.childs[i] = self.header.lastBlock ;
                            self.write_block(&rootBlock , root as usize );
                        }; 
                            ij = self.insert(key , rootBlock.childs[i] ) ; 
                        
                    }
                }
                return ij ; 
            }
            
        }
        
       
   }
   pub fn read(&mut self , root : i32 ) -> String {
        let mut message = String::new() ; 
        let mut root = self.read_block(root as usize ).unwrap(); 
       
        if root.degree != 0 {
            if root.childs[0] != 0 {
                message.push_str(&self.read(root.childs[0]));
            }
            
        println!("{:?}" , &root.values[..(root.degree )as usize ]); 
        message.push_str(&format!("{:?}\n" ,&root.values[..(root.degree )as usize])); 

         for i in 1..(root.degree - 1 ) {
            if root.childs[i as usize ] != 0 {
                 message.push_str(&self.read(root.childs[i as usize ]) );
            }
        }
        if root.childs[(root.degree - 1 ) as usize ] != 0 {
            message.push_str(&self.read(root.childs[(root.degree - 1 )as usize ])); 
        }

        
        }
        
message 

    
   }
}