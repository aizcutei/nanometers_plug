// An example for receiver
// Won't compile, just for demonstration

use interprocess::local_socket::{LocalSocketStream, NameTypeSupport};
use std::io::{prelude::*, BufReader};

#[cfg(target_os = "macos")]
pub const RING_BUFFER_SIZE: usize = 44100;

#[cfg(not(target_os = "macos"))]
pub const RING_BUFFER_SIZE: usize = 48000;

fn main() {
    let name = {
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyPaths => "/tmp/nanometers.sock",
            OnlyNamespaced | Both => "@nanometers.sock",
        }
    };

    let mut conn = LocalSocketStream::connect(name).expect("ERR: failed to connect to socket");

    let mut reader = BufReader::new(&mut stream);

    let mut buffer = [0; ((RING_BUFFER_SIZE + 1) * 4) as usize];
    let mut buffer = &mut buffer[..];

    loop {
        match reader.read(&mut buffer) {
            Ok(a) => {
                let mut buffer_f32 = buffer
                    .chunks_exact(4)
                    .map(|chunk| {
                        let mut bytes = [0; 4];
                        bytes.copy_from_slice(chunk);
                        f32::from_ne_bytes(bytes)
                    })
                    .collect::<Vec<f32>>();
            }
            Err(e) => {
                reader = BufReader::new(
                    LocalSocketStream::connect(name).context("Failed to connect to server")?,
                );
                // also can set a sleep here.
                continue;
            }
        }
    }
}
