use bitvec::prelude::*;
use image::{Pixel, RgbImage};

use crate::error::{convert_err, ErrCode, Error};

fn ceil_div(rhs: u32, lhs: u32) -> u32 {
    if rhs % lhs == 0 { rhs / lhs } else { ceil_div(rhs + 1, lhs) }
}

pub fn embed(mut img: RgbImage, payload: Vec<u8>) -> RgbImage {
    let mut text_bits: BitVec<u8, Lsb0> = BitVec::from_vec((payload.len() as u32).to_le_bytes().to_vec());
    let mut text:BitVec<u8, Lsb0> =  BitVec::from_vec(payload);
    text_bits.push(false); // pad length so that it's 33 bits long; makes it easier to extract it later
    text_bits.append(&mut text);
    let mut cursor = text_bits.iter_mut();
    for pixel in img.pixels_mut() {
        pixel.apply(|c| {
            if let Some(bit) = cursor.next() {
                c & 254 | *bit as u8
            } else {
                c
            }
        });
    }
    img
}

pub fn extract(img: RgbImage) -> Result<Vec<u8>, Error> {
    let header = img.pixels()
        .take(11)
        .flat_map(|e| e.0.to_vec())
        .map(|c| if c % 2 == 0 {false} else {true})
        .take(32)
        .collect::<BitVec<u8, Lsb0>>().into_vec();
    let len = u32::from_le_bytes(header[..4].try_into()
        .map_err(|e| convert_err(e, ErrCode::Serial))?);
    let body_len_chunks = ceil_div(len * 8, 3);

    let body = img.pixels()
        .skip(11)
        .take(body_len_chunks as usize)
        .flat_map(|e| e.0.to_vec())
        .map(|c| if c % 2 == 0 {false} else {true})
        .take(len as usize * 8)
        .collect::<BitVec<u8, Lsb0>>().into_vec();
    Ok(body)
}
