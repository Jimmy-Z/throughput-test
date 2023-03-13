### This program runs a very simple TCP throughput test with one connection and one direction:

* the server accepts one connecton and once it establishes, write 32GB zeros to it, then close.
* the client connects to the server, discard anything it receives until server closes.

To be simple the server and client is in the same program.

### There are (currently) 6 flavors:

* thread, uses native thread, `std::net` and `std::thread`.
* tokio, uses tokio, multi_thread runtime, spawns client in a separate task.
* tokio_join, uses tokio, current_thread runtime, use `tokio::join` to run both client and server in a single task.
* tokio_uring, uses tokio_uring, spawns client in a separate task.
	* in the same thread, tokio_uring doesn't have a multi_thread flavor.
* tokio_uring_mt, uses tokio_uring, spawns two tokio_uring runtimes in two native threads.
* monoio, uses monoio, with zero-copy enabled (I think?).

### Usage
* `cargo run --release --bin <flavor>`
* the monoio flavor requries `cargo +nightly run` ...

### Results
* in GB/s.
* Software:
	* Debian 11 VM in Windows 10 Hyper-V
	* kernel 5.10.0-21-amd64
	* rustc 1.70.0-nightly (7b4f48927 2023-03-12)
* Hardware: Ryzen 5 2600, DDR4-3000 32G

| flavor | 1st run | 2nd | 3rd |
| --- | --- | --- | --- |
| thread | 2.83 | 2.79 | 2.80 |
| tokio | 3.79 | 2.97 | 2.94 |
| tokio_join | 5.64 | 5.64 | 5.59 |
| tokio_uring | 2.99 | 3.01 | 2.97 |
| tokio_uring_mt | 2.49 | 2.51 | 2.41 |
| monoio | 4.68 | 4.97 | 4.70 |

Basically it's all over the place, and I don't have any explanation, try it yourself.

For reference iperf3 gets 24.5 Gbits/sec in the same setup.
