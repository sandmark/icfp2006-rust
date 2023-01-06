use std::{fs::File, io::Write};
use std::io;
use std::io::Read;
use bytes::{Buf, Bytes};
use std::env;

fn main() {
    let mut um = UM::default();
    let args: Vec<String> = env::args().collect();
    let file = args.get(1).expect("You must specify the UM binary file.");
    let buf = read_file_to_vec(file).unwrap();

    um.programs.push(buf);

    um.spin_cycle();
}

fn read_file_to_vec(path: &str) -> std::io::Result<Vec<u32>> {
    let mut file = File::open(path)?;
    let mut buf  = vec![];
    let mut vec: Vec<u32>  = vec![];

    file.read_to_end(&mut buf)?;

    let mut buf = Bytes::from(buf);

    while buf.has_remaining() {
        vec.push(buf.get_u32());
    }

    Ok(vec)
}

#[inline]
fn op_code(p: u32) -> u8 {
    (p >> 28) as u8
}

#[inline]
fn rega_offset(p: u32) -> usize {
    ((p >> 6) & 7) as usize
}

#[inline]
fn regb_offset(p: u32) -> usize {
    ((p >> 3) & 7) as usize
}

#[inline]
fn regc_offset(p: u32) -> usize {
    (p & 7) as usize
}

#[inline]
fn rego_offset(p: u32) -> usize {
    ((p >> 25) & 7) as usize
}

#[inline]
fn rego_value(p: u32) -> u32  {
    p & 0x1ffffff
}

#[derive(Debug, Default)]
struct UM {
    registers: [u32; 8],
    programs: Vec<Vec<u32>>,
    finger: usize,
    freelist: Vec<u32>,
}

impl UM {
    fn spin_cycle(&mut self) {
        let mut p: u32;
        let mut a: usize;
        let mut b: usize;
        let mut c: usize;

        macro_rules! reg {
            ($x:ident) => {
                self.registers[$x]
            }
        }

        loop {
            p = self.programs[0][self.finger];
            a = rega_offset(p);
            b = regb_offset(p);
            c = regc_offset(p);

            // println!("Register: {:?}", self.registers);
            // println!("Finger: {}, OP: {}", self.finger, op_code(p));

            match op_code(p) {
                // Conditional Move
                0 => if reg!(c) != 0 { reg!(a) = reg!(b) },

                // Array Index
                1 => reg!(a) = self.programs[reg!(b) as usize][reg!(c) as usize],

                // Array Amendment
                2 => self.programs[reg!(a) as usize][reg!(b) as usize] = reg!(c),

                // Addition
                3 => reg!(a) = reg!(b).wrapping_add(reg!(c)),

                // Multiplication
                4 => reg!(a) = reg!(b).wrapping_mul(reg!(c)),
                // Division
                5 => reg!(a) = reg!(b).wrapping_div(reg!(c)),
                // Nand
                6 => reg!(a) = !(reg!(b) & reg!(c)),
                // Halt
                7 => {
                    println!("\nUM: Halt.");
                    println!("{:?}", self.registers);
                    break;
                },
                // Allocation
                8 => {
                    let array = vec![0; reg!(c) as usize];
                    if let Some(i) = self.freelist.pop() {
                        reg!(b) = i;
                        self.programs[i as usize] = array;
                    } else {
                        reg!(b) = self.programs.len() as u32;
                        self.programs.push(array);
                    }
                },
                // Abandonment
                9 => {
                    self.freelist.push(reg!(c));
                    self.programs[reg!(c) as usize].clear();
                    self.programs[reg!(c) as usize].shrink_to_fit();
                },
                // Output
                10 => {
                    print!("{}", char::from_u32(reg!(c)).unwrap());
                    io::stdout().flush().unwrap();
                },
                // Input
                11 => {
                    let mut buf = [0_u8];
                    io::stdin().read_exact(&mut buf).unwrap();
                    reg!(c) = buf[0] as u32;
                }
                // Load Program
                12 => {
                    if reg!(b) != 0 {
                        let array = self.programs[reg!(b) as usize].clone();
                        self.programs[0] = array;
                    }
                    self.finger = reg!(c) as usize;
                    continue;
                },
                // Orthography
                13 => self.registers[rego_offset(p)] = rego_value(p),
                _ => {
                    println!("Unknown OP: {}", op_code(p));
                },
            }
            self.finger += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn make_platter(op: u8, a: u8, b: u8, c: u8) -> u32 {
        assert!(op <= 13);
        assert!(a <= 7);
        assert!(b <= 7);
        assert!(c <= 7);

        let op = (op as u32) << 28;
        let a  = (a  as u32) << 6;
        let b  = (b  as u32) << 3;
        let c  = c as u32;

        op | a | b | c
}

    #[test]
    fn test_op_code() {
        let p = make_platter(0b0101, 0, 1, 2);
        assert_eq!(0b0101, op_code(p));
    }

    #[test]
    fn test_reg_offsets() {
        let p = make_platter(0, 1, 2, 3);
        assert_eq!(1, rega_offset(p));
        assert_eq!(2, regb_offset(p));
        assert_eq!(3, regc_offset(p));
    }

    #[test]
    fn test_13_orthography() {
        let p: u32 = 0b00001110000000000000000000000001;
        assert_eq!(7, rego_offset(p));
        assert_eq!(1, rego_value(p));
    }
}
