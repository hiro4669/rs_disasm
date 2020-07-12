pub trait BinData {
    fn get_text_len(&self) -> u32;
    fn get_data_len(&self) -> u32;
    fn get_text(&self) -> Box<[u8]>;
}

pub struct MinixBinData {
    //val: u32,
    len: usize,
    data: Vec<u8>,
}

impl MinixBinData {
    pub fn new(bindata: Vec<u8>) -> Self {
        MinixBinData {
            //val:10,
            len: bindata.len(),
            data: bindata,
        }
    }

    fn read_int(buf: &[u8], idx: usize) -> u32 {
        let v1:u32 = buf[idx] as u32;
        let v2:u32 = (buf[idx+1] as u32) << 8;
        let v3:u32 = (buf[idx+2] as u32) << 16;
        let v4:u32 = (buf[idx+3] as u32) << 24;    
        v4 | v3 | v2 | v1
    }
    
}

impl BinData for MinixBinData {
    fn get_text_len(&self) -> u32 {
        //println!("len = {}", self.data.len());
        MinixBinData::read_int(&self.data,  8)
    }

    fn get_data_len(&self) -> u32 {
        MinixBinData::read_int(&self.data,  12)
    }

    fn get_text(&self) -> Box<[u8]>{
        let size: usize = self.get_text_len() as usize;
        let mut text: Vec<u8> = vec![0; size];
        let begin: usize = 0x20;        
        let s = &self.data[begin .. begin + size];
                
        // memcpy
        for (d, s) in text.iter_mut().zip(s.iter()) {*d = *s; }

        /*
        for data in text.iter() {
            print!("{:02x} ", data);
        }
        */
        text.into_boxed_slice()
    }
}
/*
pub trait BinData {
    fn get_text_len(&self) -> u32;
    //fn get_data_len(&self) -> u32;
    //fn get_text(&self) -> Vec<u8>;
}

pub struct MinixBinData {
    len: usize,
    data: Vec<u8>
}

impl MinixBinData {
    pub fn new(bindata: Vec<u8>) -> Self {
        MinixBinData {
            len: bindata.len(),
            data: bindata,            
        }
    }

    pub fn get_text_len(&self) -> u32 {
        20
    }
}

impl BinData for MinixBinData {
    fn get_text_len(&self) -> u32 {
        self.get_text_len()
    }    
}
*/