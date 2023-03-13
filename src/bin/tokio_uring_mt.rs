use std::{
	net::{IpAddr, Ipv4Addr, SocketAddr},
	thread,
	time::Instant,
};
use tokio_uring::net::{TcpListener, TcpStream};

const BLOCK_SIZE: usize = 0x8000;
const COUNT: usize = 0x100000;

const BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2501);

fn main() {
	let core_ids: [_; 2] = core_affinity::get_core_ids().unwrap()[..2].try_into().unwrap();

	let t0 = Instant::now();
	let client_jh = thread::spawn(move || {
		core_affinity::set_for_current(core_ids[0]);
		tokio_uring::start(uring_server());
	});
	let server_jh = thread::spawn(move || {
		core_affinity::set_for_current(core_ids[1]);
		tokio_uring::start(uring_cilent());
	});

	client_jh.join().unwrap();
	server_jh.join().unwrap();
	let elapsed = t0.elapsed().as_secs_f32();
	println!(
		"{:.2} seconds, {:.2} GB/s",
		elapsed,
		(BLOCK_SIZE * COUNT) as f32 / 1000_000_000f32 / elapsed
	)
}

async fn uring_cilent() {
	let client = TcpStream::connect(BIND_ADDR).await.unwrap();
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
}

async fn uring_server() {
	let server = TcpListener::bind(BIND_ADDR).unwrap();
	let server_addr = server.local_addr().unwrap();
	println!("server: listen on {}", server_addr);
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
}
