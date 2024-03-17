// An example for receiver
// Won't compile, just for demonstration
use anyhow::Context;
use interprocess::local_socket::{LocalSocketStream, NameTypeSupport};
use std::{
    backtrace,
    io::{prelude::*, BufReader},
};

#[cfg(target_os = "windows")]
pub const RING_BUFFER_SIZE: usize = 48000;

#[cfg(not(target_os = "windows"))]
pub const RING_BUFFER_SIZE: usize = 15360;

pub fn main() -> anyhow::Result<()> {
    let name = {
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => "/tmp/nanometers.sock",
            OnlyNamespaced | Both => "@nanometers.sock",
        }
    };
    let mut conn = LocalSocketStream::connect(name).context("Failed to connect to server")?;

    let mut conn = BufReader::new(conn);
    let mut buffer = [0; ((RING_BUFFER_SIZE + 1) * 4) as usize];
    let mut buf = &mut buffer[..];

    loop {
        match conn.read(&mut buf) {
            Ok(a) => {
                let mut buffer = buf
                    .chunks_exact(4)
                    .map(|chunk| {
                        let mut bytes = [0; 4];
                        bytes.copy_from_slice(chunk);
                        f32::from_ne_bytes(bytes)
                    })
                    .collect::<Vec<f32>>();
                println!("{:?}", buffer[1]);
            }
            Err(e) => {
                // println!("Error: {:?}", e);
                conn = BufReader::new(
                    LocalSocketStream::connect(name).context("Failed to connect to server")?,
                );
                continue;
            }
        }
    }
    Ok(())
}
