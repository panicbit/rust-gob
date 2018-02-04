use std::io::Read;
use std::mem::{self,size_of};
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BE;
use errors::*;
use types::TypeId;

pub trait ReadGob: Read {
    fn read_gob_u64(&mut self) -> Result<u64> {
        let byte = self.read_i8()?;

        if byte >= 0 {
            return Ok(byte as u64);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = self.read_uint::<BE>(n_bytes)?;

        Ok(bytes as u64)
    }

    fn read_gob_usize(&mut self) -> Result<usize> {
        let byte = self.read_i8()?;

        if byte >= 0 {
            return Ok(byte as usize);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<usize>() || n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = self.read_uint::<BE>(n_bytes)?;

        Ok(bytes as usize)
    }

    fn read_gob_i64(&mut self) -> Result<i64> {
        let bytes = self.read_gob_u64()?;
        let is_complement = bytes & 1 == 1;
        let bytes = (bytes >> 1) as i64;

        if is_complement {
            Ok(!bytes)
        } else {
            Ok(bytes)
        }
    }

    fn read_gob_isize(&mut self) -> Result<isize> {
        let bytes = self.read_gob_usize()? as isize;
        let is_complement = bytes & 1 == 1;
        let bytes = bytes >> 1;

        if is_complement {
            Ok(!bytes)
        } else {
            Ok(bytes)
        }
    }

    fn read_gob_bytes(&mut self) -> Result<Vec<u8>> {
        let len = self.read_gob_usize()?;
        // TODO: Allow setting maximum length to avoid malicious OOM
        let mut r = self.take(len as u64); // TODO: Fix cast
        let mut data = Vec::with_capacity(len);
        r.read_to_end(&mut data)?;

        Ok(data)
    }

    fn read_gob_f64(&mut self) -> Result<f64> {
        unsafe {
            let float: u64 = self.read_gob_u64()?;
            let float: f64 = mem::transmute(float.swap_bytes());
            Ok(float)
        }
    }

    fn read_gob_bool(&mut self) -> Result<bool> {
        self.read_gob_usize().map(|b| b != 0)
    }

    fn read_gob_type_id(&mut self) -> Result<TypeId> {
        self.read_gob_i64()
    }
}

impl<R: Read> ReadGob for R {}
