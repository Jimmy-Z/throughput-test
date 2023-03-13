
use std::time::Instant;

use monoio::{
	io::{AsyncReadRent, AsyncWriteRentExt, AsyncWriteRent},
	net::{TcpListener, TcpStream},
};

const BLOCK_SIZE: usize = 0x8000;
const COUNT: usize = 0x100000;

#[monoio::main]
async fn main() {
	let server = TcpListener::bind("127.0.0.1:0").unwrap();
	let server_addr = server.local_addr().unwrap();
	println!("server: listen on {}", server_addr);

	let client_jh = monoio::spawn(async move {
		let mut client = TcpStream::connect(server_addr).await.unwrap();
		let mut buf = Vec::with_capacity(BLOCK_SIZE);
		let mut total = 0;
		loop {
			let res;
			buf.clear();
			(res, buf) = client.read(buf).await;
			let res = res.unwrap();
			if res == 0 {
				break;
			}
			total += res;
		}
		println!("client: {} bytes received", total);
		client.shutdown().await.unwrap();
	});

	let t0 = Instant::now();
	let (mut conn, client_addr) = server.accept().await.unwrap();
	println!("server: connection from {}", client_addr);
	let mut buf = vec![0; BLOCK_SIZE];
	for _ in 0..COUNT {
		let res;
		(res, buf) = conn.write_all(buf).await;
		res.unwrap();
	}
	println!("server: all written");
	conn.shutdown().await.unwrap();

	client_jh.await;
	let elapsed = t0.elapsed().as_secs_f32();
	println!(
		"{:.2} seconds, {:.2} GB/s",
		elapsed,
		(BLOCK_SIZE * COUNT) as f32 / 1000_000_000f32 / elapsed
	)
}
