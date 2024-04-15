use super::*;
use bytes::{Buf, BufMut};

pub trait VariantEncoding: BufMut {
    /// Advance by 8.
    fn put_bool_var(&mut self, value: bool) {
        self.put_u32_le(1);
        self.put_u32_le(value as u32);
    }

    /// Advance by 8.
    fn put_u32_var(&mut self, value: u32) {
        self.put_u32_le(2);
        self.put_u32_le(value);
    }

    /// Advance by 12.
    fn put_u64_var(&mut self, value: u64) {
        self.put_u32_le(2 | (1 << 16));
        self.put_u64_le(value);
    }

    /// Advance by 8.
    fn put_f32_var(&mut self, value: f32) {
        self.put_u32_le(3);
        self.put_f32_le(value);
    }

    /// Advance by 12.
    fn put_f64_var(&mut self, value: f64) {
        self.put_u32_le(3 | (1 << 16));
        self.put_f64_le(value);
    }

    /// Advance by 8 + bytes (padded to 4).
    fn put_string_var(&mut self, value: &str) {
        self.put_u32_le(4);
        self.put_u32_le(value.len() as u32);
        self.put_slice(value.as_bytes());
        let padding = value.len().next_multiple_of(4) - value.len();
        self.put_bytes(0, padding);
    }

    /// Advance by 12.
    fn put_vec2_var(&mut self, value: Vector2<f32>) {
        self.put_u32_le(5);
        self.put_f32_le(value.x);
        self.put_f32_le(value.y);
    }

    /// Advance by 8.
    fn put_array_var(&mut self, len: usize) {
        self.put_u32_le(28);
        self.put_u32_le(len as u32);
    }

    /// Advance by 8 + bytes (padded to 4).
    fn put_bytes_var(&mut self, value: &[u8]) {
        self.put_u32_le(29);
        self.put_u32_le(value.len() as u32);
        self.put_slice(value);
        let padding = value.len().next_multiple_of(4) - value.len();
        self.put_bytes(0, padding);
    }
}
impl VariantEncoding for Vec<u8> {}

pub trait VariantDecoding: Buf {
    fn get_bool_var(&mut self) -> anyhow::Result<bool> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for bool");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 1 && flag == 0);

        Ok(self.get_u32_le() != 0)
    }

    /// Convert from 64 bits int if needed.
    fn get_u32_var(&mut self) -> anyhow::Result<u32> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for int");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 2);

        if flag == 0 {
            if self.remaining() < 4 {
                anyhow::bail!("Buffer too small for 32bits int");
            } else {
                Ok(self.get_u32_le())
            }
        } else if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for 64bits int");
        } else {
            Ok(self.get_u64_le() as u32)
        }
    }

    /// Convert from 32 bits int if needed.
    fn get_u64_var(&mut self) -> anyhow::Result<u64> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for int");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 2);

        if flag == 0 {
            if self.remaining() < 4 {
                anyhow::bail!("Buffer too small for 32bits int");
            } else {
                Ok(self.get_u32_le() as u64)
            }
        } else if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for 64bits int");
        } else {
            Ok(self.get_u64_le())
        }
    }

    /// Convert f64 to f32 if needed.
    fn get_f32_var(&mut self) -> anyhow::Result<f32> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for float");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 3);

        if flag == 0 {
            if self.remaining() < 4 {
                anyhow::bail!("Buffer too small for 32bits float");
            } else {
                Ok(self.get_f32_le())
            }
        } else if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for 64bits float");
        } else {
            Ok(self.get_f64_le() as f32)
        }
    }

    /// Convert f32 to f64 if needed.
    fn get_f64_var(&mut self) -> anyhow::Result<f64> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for float");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 3);

        if flag == 0 {
            if self.remaining() < 4 {
                anyhow::bail!("Buffer too small for 32bits float");
            } else {
                Ok(self.get_f32_le() as f64)
            }
        } else if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for 64bits float");
        } else {
            Ok(self.get_f64_le())
        }
    }

    fn get_string_var(&mut self) -> anyhow::Result<String> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for string");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 4 && flag == 0);

        let len = self.get_u32_le() as usize;
        let full_len = len.next_multiple_of(4);
        if self.remaining() < full_len {
            anyhow::bail!(
                "Buffer too small for string of length {}( has:{}, need:{})",
                len,
                self.remaining(),
                full_len
            );
        }

        let mut vec = vec![0; full_len];
        self.copy_to_slice(&mut vec);
        vec.truncate(len);
        Ok(String::from_utf8(vec)?)
    }

    fn get_vec2_var(&mut self) -> anyhow::Result<Vector2<f32>> {
        if self.remaining() < 12 {
            anyhow::bail!("Buffer too small for Vector2");
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 5 && flag == 0);

        Ok(Vector2::new(self.get_f32_le(), self.get_f32_le()))
    }

    fn get_array_var(&mut self) -> anyhow::Result<usize> {
        if self.remaining() < 8 {
            anyhow::bail!("Buffer too small for array");
        }

        // Header has godot properties which we don't care about.
        self.advance(4);

        Ok(self.get_u32_le() as usize)
    }
}
impl VariantDecoding for &[u8] {}
