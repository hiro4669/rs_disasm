use std::fs::File;
use std::io::Read;
//use binary::BinData;
use binary::BinData;

mod binary;
mod disasm;

pub fn get_type<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

fn read_int(buf: &[u8], idx: usize) -> u32 {
    let v1:u32 = buf[idx] as u32;
    let v2:u32 = (buf[idx+1] as u32) << 8;
    let v3:u32 = (buf[idx+2] as u32) << 16;
    let v4:u32 = (buf[idx+3] as u32) << 24;

    v4 | v3 | v2 | v1

}

fn f(buffer: Vec<u8>) {
    //let data = vec![0x10, 0x00, 0x00, 0x00, 0x26, 0x00, 0x00, 0x00];
    //println!("The bytes: {:?}", &data[..8]);

    let len = read_int(&buffer, 8);
    println!("len = {}", len);

    let text = &buffer[0x20 .. 0x20 + len as usize];
    println!("text len = {}", text.len());
    for i in 0 .. text.len() {
        if i > 0 && i % 16 == 0 { println!()};
        print!("{:02x} ", text[i as usize]);  
    }
    
}

fn start<T: disasm::Disasm>(mut dis: T) {
    dis.disasm();
    
}

fn main() -> std::io::Result<()>{
    let mut file = File::open("a.out")?;
    let size = file.metadata()?.len();
    //assert_eq!(198, size);
    let mut buffer = Vec::<u8>::with_capacity(size as usize);

    // read up to 10 bytes
    //let n = file.read(&mut buffer[..])?;
    //let n = file.read(&mut data)?;
    let n = file.read_to_end(&mut buffer)?;
    
    /*
    for i  in 0 .. n {
        if i > 0 && i % 16 == 0 { println!()};
        print!("{:02x} ", buffer[i as usize]);        
    }
    println!();
    */

    
    let minix_binary = binary::MinixBinData::new(buffer);
    //let text_len = minix_binary.get_text_len();
    //println!("text_len = {}", text_len);

    //minix_binary.get_text();

    let disasm = disasm::I8086Disasm::new(minix_binary);
    //disasm.disasm();
    start(disasm);


    //let mut dis = disasm::I8086Disasm::new(10);
    ////dis.disasm();
    


    
    

    Ok(())

}
