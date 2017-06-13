extern crate byteorder;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;


pub mod errors;
pub mod de;
mod types;

pub use de::Deserializer;

use std::io::{self, Read};
use std::mem::size_of;
use std::mem;
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BE;
pub use errors::*;

pub trait GobDecodable: Sized {
    fn decode<R: Read>(r: &mut R) -> Result<Self>;
}

impl GobDecodable for u64 {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let byte = r.read_i8()?;

        if byte >= 0 {
            return Ok(byte as Self);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = r.read_uint::<BE>(n_bytes)?;

        Ok(bytes as Self)
    }
}

impl GobDecodable for usize {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let byte = r.read_i8()?;

        if byte >= 0 {
            return Ok(byte as Self);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<Self>() || n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = r.read_uint::<BE>(n_bytes)?;

        Ok(bytes as usize)
    }
}

impl GobDecodable for isize {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let bytes = usize::decode(r)? as isize;
        let is_complement = bytes & 1 == 1;
        let bytes = bytes >> 1;

        if is_complement {
            Ok(!bytes)
        } else {
            Ok(bytes)
        }
    }
}

impl GobDecodable for Vec<u8> {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::decode(r)?;
        // TODO: Allow setting maximum length to avoid malicious OOM
        let mut r = r.take(len as u64); // TODO: Fix cast
        let mut data = Vec::with_capacity(len);
        r.read_to_end(&mut data)?;

        Ok(data)
    }
}

impl GobDecodable for f64 {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        unsafe {
            let float: u64 = u64::decode(r)?;
            let float: f64 = mem::transmute(float.swap_bytes());
            Ok(float)
        }
    }
}
