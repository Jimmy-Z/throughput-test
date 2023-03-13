use std::{
	io::{Read, Write},
	net::{self, TcpListener, TcpStream},
	thread,
	time::Instant,
};

// 32K block * 1M = 32G data
const BLOCK_SIZE: usize = 0x8000;
const COUNT: usize = 0x100000;

fn main() {
	let core_ids: [_; 2] = core_affinity::get_core_ids().unwrap()[..2].try_into().unwrap();
	core_affinity::set_for_current(core_ids[0]);

	let server = TcpListener::bind("127.0.0.1:0").unwrap();
	let server_addr = server.local_addr().unwrap();
	println!("server: listening on {}", server_addr);

	let client_jh = thread::spawn(move || {
		core_affinity::set_for_current(core_ids[1]);
		let mut client = TcpStream::connect(server_addr).unwrap();
		let mut buf = [0u8; BLOCK_SIZE];
		let mut total = 0;
		loop {
			let read = client.read(&mut buf).unwrap();
			if read == 0 {
				break;
			}
			total += read;
		}
		println!("client: {} bytes received", total);
		client.shutdown(net::Shutdown::Both).unwrap();
	});

	let t0 = Instant::now();
	let (mut conn, client_addr) = server.accept().unwrap();
	println!("server: connection from {}", client_addr);
	let buf = [0u8; BLOCK_SIZE];
	for _ in 0..COUNT {
		conn.write_all(&buf).unwrap();
	}
	println!("server: all written");
	conn.shutdown(net::Shutdown::Both).unwrap();

	client_jh.join().unwrap();
	let elapsed = t0.elapsed().as_secs_f32();
	println!(
		"{:.2} seconds, {:.2} GB/s",
		elapsed,
		(BLOCK_SIZE * COUNT) as f32 / 1000_000_000f32 / elapsed
	);
}
