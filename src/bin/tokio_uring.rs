use std::time::Instant;
use tokio_uring::net::{TcpListener, TcpStream};

const BLOCK_SIZE: usize = 0x8000;
const COUNT: usize = 0x100000;

fn main() {
	tokio_uring::start(uring_main());
}

async fn uring_main() {
	let server = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
	let server_addr = server.local_addr().unwrap();
	println!("server: listen on {}", server_addr);

	let client_jh = tokio_uring::spawn(async move {
		let client = TcpStream::connect(server_addr).await.unwrap();
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
		client.shutdown(std::net::Shutdown::Both).unwrap();
	});

	let t0 = Instant::now();
	let (conn, client_addr) = server.accept().await.unwrap();
	println!("server: connection from {}", client_addr);
	let mut buf = vec![0; BLOCK_SIZE];
	for _ in 0..COUNT {
		let res;
		(res, buf) = conn.write_all(buf).await;
		res.unwrap();
	}
	println!("server: all written");
	conn.shutdown(std::net::Shutdown::Both).unwrap();

	client_jh.await.unwrap();
	let elapsed = t0.elapsed().as_secs_f32();
	println!(
		"{:.2} seconds, {:.2} GB/s",
		elapsed,
		(BLOCK_SIZE * COUNT) as f32 / 1000_000_000f32 / elapsed
	)
}
