#![allow(dead_code)]
#![allow(unused_variables, unused_imports)]

extern crate serde;
extern crate bincode;


use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
use std::ops;
use std::fs;
use std::io;
use std::path::Path;
use std::io::Error;
use std::io::ErrorKind;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use self::bincode::serialize_into;
use std::mem::size_of;

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> TgaColor {
    TgaColor { bgra: [b, g, r, a], bytespp: 4 }
}

pub fn black() -> TgaColor {
    TgaColor {
        bgra: [0,0,0,255],
        bytespp: 1,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TgaHeader {
    idlength: u8,
    colormaptype: u8,
    datatypecode: u8,
    colormaporigin: i16,
    colormaplength: i16,
    colormapdepth: i8,
    x_origin: u16,
    y_origin: u16,
    width: u16,
    height: u16,
    bitsperpixel: u8,
    imagedescriptor: u8,
}

#[derive(Debug)]
pub struct TgaColor {
    pub bgra: [u8;4],
    pub bytespp: u8,
}

impl TgaColor {
    fn scale(&mut self, scale: f32) {
        self.bgra = [(self.bgra[0] as f32 * scale) as u8,
                     (self.bgra[1] as f32 * scale) as u8,
                     (self.bgra[2] as f32 * scale) as u8,
                     (self.bgra[3] as f32 * scale) as u8];
    }
}

impl ops::Mul<f32> for TgaColor {
    type Output = TgaColor;

    fn mul(self, _rhs: f32) -> TgaColor {
        TgaColor {
            bgra: [(self.bgra[0] as f32 * _rhs) as u8,
                   (self.bgra[1] as f32 * _rhs) as u8,
                   (self.bgra[2] as f32 * _rhs) as u8,
                   (self.bgra[3] as f32 * _rhs) as u8],
            bytespp: self.bytespp,
        }
    }
}

#[derive(Debug)]
pub struct TgaImage {
    buffer: Box<[u8]>,
    pub width: u16,
    pub height: u16,
    bytespp: u8,
}

impl fmt::Display for TgaImage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "TgaImage {}x{}, {} bpp, {} bytes", self.width, self.height, self.bytespp, self.buffer.len())
    }
}

fn load_rle_data(data: &[u8], width: u16, height: u16, bytespp: u8) -> io::Result<Box<[u8]>> {
    let pixelcount = width as u64 * height as u64;
    let mut currentpixel: u64 = 0;
    let mut currentbyte: usize = 0;

    let datasize = pixelcount as usize * bytespp as usize;
    let mut buffer: Box<[u8]> = vec![0; datasize].into_boxed_slice();

    let mut iter = data.iter().cloned();

    while currentpixel < pixelcount {
        let chunkheader = iter.next().unwrap();
        if chunkheader < 128 {
            for _ in 0 .. chunkheader+1 {
                for _ in 0 .. bytespp {
                    buffer[currentbyte] = iter.next().expect(format!("currentbyte: {}", currentbyte).as_str());
                    currentbyte += 1;
                }
                currentpixel += 1;
                if currentpixel > pixelcount {
                    return Err(Error::new(ErrorKind::Other, "Too many pixels"))

                }
            }
        } else {
            let mut chunk= vec![0; bytespp as usize];
            for i in 0 .. bytespp as usize {
                chunk[i] = iter.next().unwrap();
            }

            for _ in 0 .. chunkheader-127 {
                for t in 0 .. bytespp as usize {
                    buffer[currentbyte] = chunk[t];
                    currentbyte += 1;
                }
                currentpixel +=1;
                if currentpixel > pixelcount {
                    return Err(Error::new(ErrorKind::Other, "Too many pixels read"));
                }
            }
        }
    }

    Ok(buffer)
}

impl TgaImage {
    pub fn empty() -> TgaImage {
        TgaImage { buffer: vec![].into_boxed_slice(), width: 0, height: 0, bytespp: 0 }
    }

    pub fn new(width: u16, height: u16, bytespp: u8) -> TgaImage {
        let play = "s";
        let data_size: usize = width as usize * height as usize * bytespp as usize;
        TgaImage { buffer: vec![0; data_size].into_boxed_slice(), width, height, bytespp }
    }


    pub fn read_tga_file<P: AsRef<Path>>(path: P) -> io::Result<TgaImage> {
        let data = fs::read(path)?;

        let header_size = size_of::<TgaHeader>();
        let header: TgaHeader = deserialize(&data[..header_size]).unwrap();

        let width = header.width;
        let height = header.height;
        if width == 0 || height == 0 {
            return Err(Error::new(io::ErrorKind::Other, "bad dimensions" ))
        }

        let bytespp = header.bitsperpixel / 8;

        if bytespp != 1 && bytespp != 3 && bytespp != 4 {
            return Err(Error::new(io::ErrorKind::Other, "unsupported bitspp"))
        }

        let buffer: io::Result<Box<[u8]>> =
        if header.datatypecode == 3 || header.datatypecode == 2 {
            //just read the data
            Ok(data.into_boxed_slice())
        } else if header.datatypecode == 10 || header.datatypecode == 11 {
            load_rle_data(&data[header_size..], width, height, bytespp)
        } else {
            Err(Error::new(io::ErrorKind::InvalidData, "unsupported data type"))
        };

        let mut image = TgaImage { buffer: buffer.unwrap(), width, height, bytespp };

        if (header.imagedescriptor & 0x20) == 0 {
            image.flip_vertically();
        }
        if (header.imagedescriptor & 0x10) != 0 {
            image.flip_horizontally();
        }
        Ok(image)

    }

    pub fn write_tga_file<P: AsRef<Path>>(&self, path: P, rle: bool) -> io::Result<()> {
        let developer_area_ref: [u8; 4] = [0; 4];
        let extension_area_ref: [u8; 4] = [0; 4];
        let footer: &[u8; 18] = b"TRUEVISION-XFILE.\0";

        let mut file = File::create(path)?;

        let datatypecode =
            if self.bytespp == 1 {
                if rle { 11 } else { 3 }
            } else {
                if rle { 10 } else { 2 }
            };

        let header = TgaHeader {
            idlength: 0,
            bitsperpixel: self.bytespp * 8,
            width: self.width,
            height: self.height,
            datatypecode,
            colormaporigin: 0,
            colormaplength: 0,
            colormapdepth: 0,
            x_origin: 0,
            imagedescriptor: 0x20, // top-left origin
            colormaptype: 0,
            y_origin: 0
        };

        serialize_into(&file, &header).unwrap();

        if !rle {
            file.write_all(self.buffer.as_ref())?;
        } else {
            self.unload_rle_data(&file)?;
        }

        file.write_all(&developer_area_ref)?;
        file.write_all(&extension_area_ref)?;
        file.write_all(footer)?;
        Ok(())
    }

    fn unload_rle_data(&self, writer: &Write) -> io::Result<()> {
        unimplemented!()
    }

    pub fn flip_horizontally(&mut self) {
        let half = self.width / 2;

        for i in 0..half {
            for j in 0..self.height {
                let c1 = self.get(i, j);
                let c2 = self.get(self.width - 1 - i, j);

                self.set(i, j, c2);
                self.set(self.width - 1 - i, j, c1);
            }
        }
    }

    pub fn flip_vertically(&mut self) {
        let half = self.height / 2;

        for i in 0..self.width {
            for j in 0..half {
                let c1 = self.get(i, j);
                let c2 = self.get(i, self.height - 1 - j);

                self.set(i, j, c2);
                self.set(i, self.height - 1 - j, c1);
            }
        }
    }

    pub fn get(&self, x: u16 , y:u16 ) -> TgaColor {
        let x = x as usize;
        let y = y as usize;
        let width = self.width as usize;
        let bpp = self.bytespp as usize;
        let offset = (x + (y * width)) * bpp;

        let b = self.buffer[offset];
        let g = if self.bytespp > 1 { self.buffer[offset + 1] } else { 0 };
        let r = if self.bytespp > 2 { self.buffer[offset + 2] } else { 0 };
        let a = if self.bytespp > 3 { self.buffer[offset + 3] } else { 0 };

        TgaColor { bgra: [b, g, r, a], bytespp: self.bytespp }
    }

    pub fn set(&mut self, x: u16, y:u16,  color: TgaColor) -> io::Result<()> {
        let x = x as usize;
        let y = y as usize;
        let width = self.width as usize;
        let bpp = self.bytespp as usize;

        let offset = (x + (y * width)) * bpp;
        let offset = offset as usize;
        let iter = color.bgra.iter();
        for i in 0..self.bytespp {
            self.buffer[offset + i as usize] = color.bgra[i as usize];
        }
        Ok(())
    }

    pub fn scale(&mut self, x: i32, y:i32) {
        unimplemented!()
    }

    pub fn clear(&mut self) {
        unimplemented!()
    }
}
