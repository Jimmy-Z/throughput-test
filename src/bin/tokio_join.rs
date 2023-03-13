use std::time::Instant;
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
};

const BLOCK_SIZE: usize = 0x8000;
const COUNT: usize = 0x100000;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
	let server_addr = server.local_addr().unwrap();
	println!("server: listening on {}", server_addr);

	let t0 = Instant::now();
	let _ = tokio::join!(
		async move {
			let mut client = TcpStream::connect(server_addr).await.unwrap();
			let mut buf = [0u8; BLOCK_SIZE];
			let mut total = 0;
			loop {
				let read = client.read(&mut buf).await.unwrap();
				if read == 0 {
					break;
				}
				total += read;
			}
			println!("client: {} bytes received", total);
			client.shutdown().await.unwrap();
		},
		async move {
			let (mut conn, client_addr) = server.accept().await.unwrap();
			println!("server: connection from {}", client_addr);
			let buf = [0u8; BLOCK_SIZE];
			for _ in 0..COUNT {
				conn.write_all(&buf).await.unwrap();
			}
			println!("server: all written");
			conn.shutdown().await.unwrap();
		}
	);

	let elapsed = t0.elapsed().as_secs_f32();
	println!(
		"{:.2} seconds, {:.2} GB/s",
		elapsed,
		(BLOCK_SIZE * COUNT) as f32 / 1000_000_000f32 / elapsed
	)
}
