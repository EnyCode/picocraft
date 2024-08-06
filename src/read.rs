use embassy_net::tcp::TcpReader;
use heapless::{String, Vec};

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
    // TODO: fix max size
    async fn read_string(&mut self) -> String<128>;
    // TODO: add more types
    async fn read_varint(&mut self) -> i32;
    async fn read_varlong(&mut self) -> i64;
}

macro_rules! impl_read {
    ($ty:ty, $read:ident, $size:expr) => {
        async fn $read(&mut self) -> $ty {
            let mut buf = [0; $size];
            self.read(&mut buf).await.unwrap();
            <$ty>::from_be_bytes(buf)
        }
    };
}

impl ReadExtension for TcpReader<'_> {
    impl_read!(i8, read_i8, 1);
    impl_read!(u8, read_u8, 1);
    impl_read!(i16, read_i16, 2);
    impl_read!(u16, read_u16, 2);
    impl_read!(i32, read_i32, 4);
    impl_read!(i64, read_i64, 8);
    impl_read!(f32, read_f32, 4);
    impl_read!(f64, read_f64, 8);

    async fn read_bool(&mut self) -> bool {
        self.read_u8().await != 0
    }

    async fn read_string(&mut self) -> String<128> {
        let len = self.read_varint().await as usize;
        let mut buf: Vec<u8, 128> = Vec::new();
        self.read(&mut buf[..len]).await.unwrap();
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
