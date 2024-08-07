use alloc::string::String;
use embassy_net::tcp::TcpWriter;

pub trait WriteExtension {
    async fn write_i8(&mut self, value: i8);
    async fn write_u8(&mut self, value: u8);
    async fn write_i16(&mut self, value: i16);
    async fn write_u16(&mut self, value: u16);
    async fn write_i32(&mut self, value: i32);
    async fn write_i64(&mut self, value: i64);
    async fn write_f32(&mut self, value: f32);
    async fn write_f64(&mut self, value: f64);
    async fn write_bool(&mut self, value: bool);
    async fn write_string(&mut self, value: String);
    async fn write_varint(&mut self, value: i32);
    async fn write_varlong(&mut self, value: i64);
}

macro_rules! impl_write {
    ($ty:ty, $write:ident) => {
        async fn $write(&mut self, value: $ty) {
            self.write(&value.to_be_bytes()).await.unwrap();
        }
    };
}

impl WriteExtension for TcpWriter<'_> {
    impl_write!(i8, write_i8);
    impl_write!(u8, write_u8);
    impl_write!(i16, write_i16);
    impl_write!(u16, write_u16);
    impl_write!(i32, write_i32);
    impl_write!(i64, write_i64);
    impl_write!(f32, write_f32);
    impl_write!(f64, write_f64);

    async fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 }).await;
    }

    async fn write_string(&mut self, value: String) {
        self.write_varint(value.len() as i32).await;
        self.write(value.as_bytes()).await.unwrap();
    }

    async fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut byte = (value & 0b0111_1111) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0b1000_0000;
            }
            self.write_u8(byte).await;
            if value == 0 {
                break;
            }
        }
    }

    async fn write_varlong(&mut self, value: i64) {
        todo!()
    }
}
