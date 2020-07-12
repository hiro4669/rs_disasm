use super::binary::*;

//トレイトはジェネリクスじゃない
pub trait Disasm {
    fn disasm(&mut self);
}

// 構造体はジェネリクス
pub struct I8086Disasm<T> {
    binary: T,
    text: Option<Box<[u8]>>,
    pc: usize,
}

// 定義にジェネリクスが含まれるため，impl<T>とする必要がある．T: トレイト境界でもいい
impl<T> I8086Disasm<T> {
    pub fn new(bindata: T) -> Self {
        I8086Disasm {
            binary: bindata,
            text: None,
            pc: 0,
        }
    }

    fn fetch(&mut self, opcode: &mut OpCode) -> u8 {
        if let Some(ary) = &self.text {           
            let op = ary[self.pc];
            opcode.add_raw_data(op);
            self.pc += 1;
            op
        } else {
            0
        }
    }

    fn fetch2(&mut self, opcode: &mut OpCode) -> u16 {
        if let Some(ary) = &self.text {
            //println!("fetch2");
            //println!("{:02x}", ary[self.pc]);
            opcode.add_raw_data(ary[self.pc]);
            let v1 = ary[self.pc] as u16;            
            self.pc += 1;
            //println!("{:02x}", ary[self.pc]);
            opcode.add_raw_data(ary[self.pc]);
            let v2 = (ary[self.pc] as u16 ) << 8;
            self.pc += 1;            
            v2 | v1
        } else {
            0
        }        
    }

    fn set_data(&mut self, opcode: &mut OpCode) {
        //println!("w = {}", opcode.w);
        match opcode.w {
            0 => opcode.data = self.fetch(opcode) as u16,
            1 => opcode.data = self.fetch2(opcode),
            _ => std::process::exit(1),
        }        
    }

    fn set_sdata(&mut self, opcode: &mut OpCode) {
        match (opcode.s, opcode.w) {
            (0, 1) => {
                opcode.data = self.fetch2(opcode);
            }
            (1, 1) => {                
                let data_i8 = self.fetch(opcode) as i8;
                let data = data_i8 as i16;
                opcode.data = data as u16;
            }
            _ => std::process::exit(1)
        }
    }

    fn set_mrr(&mut self, opcode: &mut OpCode) {
        let mrr = self.fetch(opcode);
        opcode.md  = (mrr >> 6) & 3;
        opcode.reg = (mrr >> 3) & 7;
        opcode.rm  = (mrr >> 0) & 7;
        self.resolv_disp(opcode);
    }

    fn resolv_disp(&mut self, opcode: &mut OpCode) {
        match opcode.md {
            0 => {
                if opcode.rm == 6 {
                    let disp = self.fetch2(opcode);
                    opcode.set_disp(disp as i16);
                } else {
                    opcode.set_disp(0);
                }
            }
            1 => {
                let disp_i8 = self.fetch(opcode) as i8;
                opcode.set_disp(disp_i8 as i16);
            }
            2 => {
                let disp = self.fetch2(opcode);
                opcode.set_disp(disp as i16);
            }
            3 => { },
            _ => {
                std::process::exit(1);
            }
        }
    }

    fn get_data_str(opcode: &OpCode) -> String {        
        let ret = match opcode.w {
            0 => String::from(&format!("{:02x}", opcode.data)),
            1 => String::from(&format!("{:04x}", opcode.data)),
            _ => std::process::exit(1),
        };
        ret
    }

    fn get_sdata_str(opcode: &OpCode) -> String {
        let ret = match (opcode.s, opcode.w) {
            (0, 1) => 
                String::from(&format!("{:04x}", opcode.data)),
            (1, 1) => 
                String::from(&format!("{:02x}", opcode.data)),           
            _ => std::process::exit(1)
        };
        ret
    }

    fn get_effective_addr(opcode: &OpCode) -> String {        
        let mem = match opcode.rm {
            0 => "bx+si",
            1 => "bx+di",
            2 => "bp+si",
            3 => "bp+di",
            4 => "si",
            5 => "di",
            6 => { 
                match opcode.md {
                    0 => "",
                    _ => "bp",
                }
            },
            7 => "bx",
            _ => std::process::exit(1),        
        };

        //let rmstr;
        if (opcode.disp != 0) && opcode.md == 0 {
            let rmstr = &format!("[{:04x}]", opcode.disp);
            String::from(rmstr)
        } else if opcode.disp != 0 {
            if opcode.disp < 0 {
                let rmstr = &format!("[{}{}]", mem, opcode.disp);
                String::from(rmstr)
            } else {
                let rmstr = &format!("[{}+{}]", mem, opcode.disp);
                String::from(rmstr)
            }
        } else {
            let rmstr = &format!("[{}]", mem);
            String::from(rmstr)
        }        
    }

    fn get_reg_str(w: u8, reg: u8) -> String {
        let rval = match (w, reg) {
            (0, 0) => "al".to_string(),
            (0, 1) => "cl".to_string(),
            (0, 2) => "dl".to_string(),
            (0, 3) => "bl".to_string(),
            (0, 4) => "ah".to_string(),
            (0, 5) => "ch".to_string(),
            (0, 6) => "dh".to_string(),
            (0, 7) => "bh".to_string(),
            (1, 0) => "ax".to_string(),
            (1, 1) => "cx".to_string(),
            (1, 2) => "dx".to_string(),
            (1, 3) => "bx".to_string(),
            (1, 4) => "sp".to_string(),
            (1, 5) => "bp".to_string(),
            (1, 6) => "si".to_string(),
            (1, 7) => "di".to_string(),
            _      => std::process::exit(0),
        };
        rval
    }

    fn get_pc_str(prev_pc: usize) -> String {
        let s = &format!("{:04x}: ", prev_pc);
        String::from(s)
    }

    fn get_rawdata_str(opcode: &OpCode) -> String {
        let mut line = String::default();
        for i in 0 .. opcode.raw_len {
            line.push_str(&format!("{:02x}", opcode.raw_data[i]));
        }
        for _ in (opcode.raw_len*2) .. 14 {
            line.push_str(" ");
        }
        line
    }

    fn dump_add(opcode: &OpCode, prev_pc: usize) {
        let mut line = String::default();
        line.push_str(&I8086Disasm::<MinixBinData>::get_pc_str(prev_pc));
        line.push_str(&I8086Disasm::<MinixBinData>::get_rawdata_str(opcode));        
        line.push_str("add ");
        let reg_str = &I8086Disasm::<MinixBinData>::get_reg_str(opcode.w, opcode.reg);
        let ef_addr = &I8086Disasm::<MinixBinData>::get_effective_addr(opcode);
        //println!("{}", ef_addr);
        match opcode.d {
            0 => {
                line.push_str(ef_addr);
                line.push_str(", ");
                line.push_str(reg_str);
            }
            1 => {
                line.push_str(reg_str);
                line.push_str(", ");
                line.push_str(ef_addr);
            }
            _ => std::process::exit(1)
        }
        println!("{}", line);
    }

    fn dump_sub(opcode: &OpCode, prev_pc: usize) {
        let mut line = String::default();
        line.push_str(&I8086Disasm::<MinixBinData>::get_pc_str(prev_pc));
        line.push_str(&I8086Disasm::<MinixBinData>::get_rawdata_str(opcode));        
        line.push_str("sub ");
        let ef_addr = &I8086Disasm::<MinixBinData>::get_effective_addr(opcode);
        line.push_str(ef_addr);
        line.push_str(", ");
        line.push_str(&I8086Disasm::<MinixBinData>::get_sdata_str(opcode));
        println!("{}", line);
    }

    fn dump_mov(opcode: &OpCode, prev_pc: usize) {
        let mut line = String::default();
        //let s = &I8086Disasm::<MinixBinData>::get_pc_str(prev_pc);
        line.push_str(&I8086Disasm::<MinixBinData>::get_pc_str(prev_pc));
        line.push_str(&I8086Disasm::<MinixBinData>::get_rawdata_str(opcode));        
        line.push_str("mov ");
        line.push_str(&I8086Disasm::<MinixBinData>::get_reg_str(opcode.w, opcode.reg));
        line.push_str(", ");
        line.push_str(&I8086Disasm::<MinixBinData>::get_data_str(opcode));        
        println!("{}", line);        
    }

    fn dump_int(opcode: &OpCode, prev_pc: usize) {
        let mut line = String::default();        
        line.push_str(&I8086Disasm::<MinixBinData>::get_pc_str(prev_pc));
        line.push_str(&I8086Disasm::<MinixBinData>::get_rawdata_str(opcode));  
        line.push_str("int ");
        line.push_str(&I8086Disasm::<MinixBinData>::get_data_str(opcode));
        println!("{}", line); 
    }

    fn dump_undefined(opcode: &OpCode, prev_pc: usize) {
        let mut line = String::default();        
        line.push_str(&I8086Disasm::<MinixBinData>::get_pc_str(prev_pc));
        line.push_str(&I8086Disasm::<MinixBinData>::get_rawdata_str(opcode));  
        line.push_str("(undefined)");
        println!("{}", line); 
    }

    
}


struct OpCode {
    pub s: u8,
    pub d: u8,
    pub w: u8,
    pub md: u8,
    pub rm: u8,
    pub reg: u8,
    pub data: u16,
    raw_len: usize,
    raw_data: [u8; 20],
    disp: i16,

}

impl OpCode {
    fn new() -> Self {
        OpCode {
            s:    0,
            d:    0,
            w:    0,
            md:   0,
            rm:   0,
            reg:  0,
            data: 0,
            raw_len: 0,
            raw_data: [0; 20],
            disp: 0,
        }
    }
    fn clear(&mut self) {
        self.d = 0; self.w = 0; self.md = 0; self.reg = 0; self.data = 0;
        self.raw_len = 0;
    }

    fn add_raw_data(&mut self, d: u8) {
        self.raw_data[self.raw_len] = d;
        self.raw_len += 1;
    }

    fn set_disp(&mut self, disp: i16) {
        self.disp = disp;
    }
    
}

// 構造体定義にジェネリクスが含まれるため，impl<T>とする必要がある．
// 多くの場合メソッドではデータを扱うので，トレイと境界が必要になる
impl<T: BinData> Disasm for I8086Disasm<T> {

    

    fn disasm(&mut self) {
        println!("exec disasm");
        //println!("{}", self.binary.get_text_len());
        let text = self.binary.get_text();
        self.text = Some(text);

        //let mut val;
        //val = self.fetch();
        //println!("val {:02x}", val);
        //val = self.fetch2();
        //println!("val {:04x}", val);
        let mut opcode = OpCode::new();
        loop {
            let prev_pc = self.pc;
            let op = self.fetch(&mut opcode);            
            match op {
                0x00 ..= 0x03 => {
                    if self.pc == self.binary.get_text_len() as usize {
                        I8086Disasm::<MinixBinData>::dump_undefined(&opcode, prev_pc);
                        break;
                    }
                    opcode.w = op & 1;
                    opcode.d = (op >> 1) & 1;
                    self.set_mrr(&mut opcode);
                    I8086Disasm::<MinixBinData>::dump_add(&opcode, prev_pc);
                    //std::process::exit(1);

                }
                0x80 ..= 0x83 => {                    
                    opcode.s = (op >> 1) & 1;
                    opcode.w = op & 1;
                    self.set_mrr(&mut opcode);
                    self.set_sdata(&mut opcode);
                    I8086Disasm::<MinixBinData>::dump_sub(&opcode, prev_pc);
                    //std::process::exit(1);
                }
                0xb0 ..= 0xbf => {               
                    opcode.w = (op >> 3) & 1;
                    opcode.reg = op & 7;
                    self.set_data(&mut opcode);

                    I8086Disasm::<MinixBinData>::dump_mov(&opcode, prev_pc);                    
                }
                0xcd => {
                    //println!("int");
                    self.set_data(&mut opcode);
                    I8086Disasm::<MinixBinData>::dump_int(&opcode, prev_pc);
                    //std::process::exit(1);
                }
                _ => { 
                    println!("unknown operator");
                    std::process::exit(1);
                }
            }


            if self.pc == self.binary.get_text_len() as usize {
                break;
            }
            opcode.clear();    
        }    
    }
}
