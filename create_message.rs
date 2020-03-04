//#![warn(rust_2018_idioms)]
/*
Copyright 2020 iljah

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use chrono::{Duration, NaiveDateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use sha2::{Digest, Sha512};

/*TODO: use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};*/

use std::{env, iter, process::exit};

static DT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

/*#[tokio::main]
async*/ fn main() {
	let temp: Vec<String> = env::args().collect();
	let mut args: Vec<&str> = Vec::<&str>::new();
	for i in 0..temp.len() {
		args.push(&temp[i])
	}

	if args.len() < 2 {
		println!("Usage: {} message [min PoW]", args[0]);
		println!("Prints message with required proof of work and timestamp to standard output for sending to a2amd");
		println!("Example: {} hello_world | socat -t 2 STDIO TCP4:127.0.0.1:1234", args[0]);
		exit(0);
	}

	let min_pow: i64;
	if args.len() > 2 {
		min_pow = args[2].parse::<i64>().unwrap()
	} else {
		min_pow = 1
	}

	let message = args[1].to_string();
	let timestamp = NaiveDateTime::from_timestamp(
		// prevent discovery of client's current time?
		(Utc::now() - Duration::milliseconds((60000. * rand::random::<f64>()) as i64)).timestamp(),
		0
	).format(DT_FORMAT).to_string();

	let mut rng = thread_rng();
	loop {
		let nonce: String = iter::repeat(()).map(|()| rng.sample(Alphanumeric)).take((min_pow / 4) as usize + 1).collect();
		let pow = get_leading_zeros(get_hash(&nonce, &timestamp, &message));
		if pow >= min_pow {
			/*TODO: send directly to server instead of printing
			let mut stream = TcpStream::connect("127.0.0.1:1234".to_string()).await.expect("asdf");
			tokio::spawn(async move {
				let mut lines = Framed::new(stream, LinesCodec::new());
				lines.send(format!("set {} {} {}", &nonce, &timestamp, &message).to_string()).await;
				let result = lines.next().await;
				match result {
					Some(line) => println!("{:?}", line),
					None => panic!()
				}
			});*/
			println!("set {} {} {}", &nonce, &timestamp, &message);
			break
		}
	}
}

// TODO: merge with identical code in a2amd.rs
fn get_hash(nonce: &String, timestamp: &String, payload: &String) -> [u8; 64] {
	let mut hasher = Sha512::new();
	hasher.input(nonce);
	hasher.input(timestamp);
	hasher.input(payload);
	let result = hasher.result();
	let mut r64: [u8; 64] = [0; 64];
	for i in 0..64 {
		r64[i] = result[i]
	}
	return r64;
}

fn get_leading_zeros(arr: [u8; 64]) -> i64 {
	let mut pow: i64 = 0;
	for i in 0..64 {
		if arr[i] < 128 {pow += 1} else {break}
		if arr[i] < 64 {pow += 1} else {break}
		if arr[i] < 32 {pow += 1} else {break}
		if arr[i] < 16 {pow += 1} else {break}
		if arr[i] < 8 {pow += 1} else {break}
		if arr[i] < 4 {pow += 1} else {break}
		if arr[i] < 2 {pow += 1} else {break}
		if arr[i] < 1 {pow += 1} else {break}
	}
	return pow;
}
