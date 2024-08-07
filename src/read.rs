use alloc::{boxed::Box, string::String, vec::Vec};
use embassy_net::tcp::TcpReader;

/// List of types is taken from [wiki.vg](https://wiki.vg/Protocol#Data_types)
pub trait ReadExtension {
    async fn read_i8(&mut self) -> i8;
    async fn read_u8(&mut self) -> u8;
    async fn read_i16(&mut self) -> i16;
    async fn read_u16(&mut self) -> u16;
    async fn read_i32(&mut self) -> i32;
    async fn read_i64(&mut self) -> i64;
    async fn read_f32(&mut self) -> f32;
    async fn read_f64(&mut self) -> f64;
    async fn read_bool(&mut self) -> bool;
    async fn read_string(&mut self) -> String;
    // TODO: add more types
    async fn read_varint(&mut self) -> i32;
    async fn read_varlong(&mut self) -> i64;
}

macro_rules! impl_tcp_read {
    ($ty:ty, $read:ident, $size:expr) => {
        async fn $read(&mut self) -> $ty {
            let mut buf = [0; $size];
            self.read(&mut buf).await.unwrap();
            <$ty>::from_be_bytes(buf)
        }
    };
}

impl ReadExtension for TcpReader<'_> {
    impl_tcp_read!(i8, read_i8, 1);
    impl_tcp_read!(u8, read_u8, 1);
    impl_tcp_read!(i16, read_i16, 2);
    impl_tcp_read!(u16, read_u16, 2);
    impl_tcp_read!(i32, read_i32, 4);
    impl_tcp_read!(i64, read_i64, 8);
    impl_tcp_read!(f32, read_f32, 4);
    impl_tcp_read!(f64, read_f64, 8);

    async fn read_bool(&mut self) -> bool {
        self.read_u8().await != 0
    }

    async fn read_string(&mut self) -> String {
        let len = self.read_varint().await as usize;
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        self.read(&mut buf).await.unwrap();
        String::from_utf8(buf).unwrap()
    }

    async fn read_varint(&mut self) -> i32 {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.read_u8().await;
            result |= ((byte & 0b0111_1111) as i32) << shift;
            if shift == 35 {
                break;
            }
            if byte & 0b1000_0000 == 0 {
                // TODO: add errors
                break;
            }
            shift += 7;
        }
        result
    }

    async fn read_varlong(&mut self) -> i64 {
        todo!();
    }
}

pub struct Slice {
    buf: Box<[u8]>,
    pos: usize,
}

impl Slice {
    pub fn new(buf: Box<[u8]>) -> Slice {
        log::info!("{:?}", buf);
        Slice { buf, pos: 0 }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<(), ()> {
        let len = buf.len();
        if len < self.buf.len() - self.pos {
            buf.clone_from_slice(&self.buf[self.pos..self.pos + len]);
            self.pos += len;
            Ok(())
        } else {
            Err(())
        }
    }
}

macro_rules! impl_slice_read {
    ($ty:ty, $read:ident, $size:expr) => {
        async fn $read(&mut self) -> $ty {
            let mut buf = [0; $size];
            self.read(&mut buf).await.unwrap();
            <$ty>::from_be_bytes(buf)
        }
    };
}

impl ReadExtension for Slice {
    impl_slice_read!(i8, read_i8, 1);
    impl_slice_read!(u8, read_u8, 1);
    impl_slice_read!(i16, read_i16, 2);
    impl_slice_read!(u16, read_u16, 2);
    impl_slice_read!(i32, read_i32, 4);
    impl_slice_read!(i64, read_i64, 8);
    impl_slice_read!(f32, read_f32, 4);
    impl_slice_read!(f64, read_f64, 8);

    async fn read_bool(&mut self) -> bool {
        self.read_u8().await != 0
    }

    async fn read_string(&mut self) -> String {
        let len = self.read_varint().await as usize;
        let mut buf = alloc::vec::Vec::with_capacity(len);
        unsafe { buf.set_len(len) }
        self.read(&mut buf).await.unwrap();
        log::info!("len {} string {:?}", len, buf);

        String::from_utf8(buf).unwrap()
    }

    async fn read_varint(&mut self) -> i32 {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.read_u8().await;
            result |= ((byte & 0b0111_1111) as i32) << shift;
            if shift == 35 {
                break;
            }
            if byte & 0b1000_0000 == 0 {
                // TODO: add errors
                break;
            }
            shift += 7;
        }
        result
    }

    async fn read_varlong(&mut self) -> i64 {
        todo!();
    }
}
