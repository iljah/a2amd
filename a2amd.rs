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
use futures::SinkExt;
use hex::decode;
use log::{debug, error, info, LevelFilter, trace, warn};
use math::round::ceil;
use sha2::{Digest, Sha512};
use simple_logging::log_to_stderr;
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

use std::{env, thread, time};
use std::collections::HashMap;
use std::error::Error;
use std::process::exit;
use std::sync::{Arc, Mutex};

static DT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

struct Message {
	nonce: String,
	timestamp: String, // in DT_FORMAT
	payload: String
}

impl Message {
	fn size(&self) -> usize {
		return self.nonce.len() + self.timestamp.len() + self.payload.len()
	}
}

struct Database {
	map: Mutex<HashMap<String, Message>>, // key = sha512 of message
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let temp: Vec<String> = env::args().collect();
	let mut args: Vec<&str> = Vec::<&str>::new();
	for i in 0..temp.len() {
		args.push(&temp[i])
	}

	if args.len() > 3 {
		let _ = match args[3] {
			"trace" => log_to_stderr(LevelFilter::Trace),
			"debug" => log_to_stderr(LevelFilter::Debug),
			"info" => log_to_stderr(LevelFilter::Info),
			"warn" => log_to_stderr(LevelFilter::Warn),
			"error" => log_to_stderr(LevelFilter::Error),
			_ => log_to_stderr(LevelFilter::Off)
		};
	}
	debug!("log level: {}", args[3]);

	trace!("{} command line arguments:", temp.len());
	for i in 0..args.len() {
		trace!("{}", args[i]);
	}

	let addr: String;
	if args.len() > 1 {
		if args[1].contains("help") {
			println!("Usage: {} [address:port to listen] [max db size] [log level]", args[0]);
			println!("Log levels: error warn info debug trace");
			println!("Stores in memory at most max db size number of bytes");
			println!("Stored messages are prioritized by their size, age and attached proof of work");
			exit(0);
		}
		if args[1] != "127.0.0.1:1234" {
			warn!("binding to non-default address {}", args[1]);
		}
		addr = args[1].to_string()
	} else {
		addr = "127.0.0.1:1234".to_string()
	}
	debug!("addr: {}", addr);

	let max_db_size: usize;
	if args.len() > 2 {
		max_db_size = args[2].parse::<usize>().unwrap()
	} else {
		max_db_size = 200
	}
	debug!("max_db_size: {}", max_db_size);

    let mut listener = TcpListener::bind(&addr).await?;
	info!("Listening on: {}", addr);

    let initial_db = HashMap::new();
    let db = Arc::new(Database{map: Mutex::new(initial_db)});

    loop {
        match listener.accept().await {
            Err(e) => println!("error accepting socket: {:?}", e),
            Ok((socket, addr)) => {
				debug!("{} connected", addr);
                let db = db.clone();

                tokio::spawn(async move {
                    let mut lines = Framed::new(socket, LinesCodec::new());

                    while let Some(result) = lines.next().await {
                        match result {
                            Ok(line) => {
                                let response = handle_request(&line, &db, max_db_size);
                                if let Err(e) = lines.send(response).await {
									info!("error on sending response; error = {:?}", e);
                                }
                            }
                            Err(e) => {info!("error on decoding from socket; error = {:?}", e);}
                        }
                    }
					debug!("{} disconnected", addr);
                });
            }
        }
    }
}

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

// in [222, 173, 190, 239, ...] out "deadbeef..."
fn bytes_to_hex(a: [u8; 64]) -> String {
	return format!("{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}{:0>2X}", a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11], a[12], a[13], a[14], a[15], a[16], a[17], a[18], a[19], a[20], a[21], a[22], a[23], a[24], a[25], a[26], a[27], a[28], a[29], a[30], a[31], a[32], a[33], a[34], a[35], a[36], a[37], a[38], a[39], a[40], a[41], a[42], a[43], a[44], a[45], a[46], a[47], a[48], a[49], a[50], a[51], a[52], a[53], a[54], a[55], a[56], a[57], a[58], a[59], a[60], a[61], a[62], a[63])
}

// in "deadbeef..." out [222, 173, 190, 239, ...]
fn hex_to_bytes(s: &String) -> [u8; 64] {
	let mut a: [u8; 64] = [0; 64];
	let bytes = match decode(s) {
		Ok(bytes) => bytes,
		Err(e) => panic!("{}, {}", e, s)
	};
	if bytes.len() != 64 {
		error!("{}", s);
		panic!()
	}
	for i in 0..64 {
		a[i] = bytes[i]
	}
	return a
}

// decreases given original proof of work based on message size (bytes)
// and difference between current time and message timestamp (seconds)
fn pow_decay(pow: i64, size: usize, time_diff: i64) -> f64 {
	let mut new_pow = pow as f64;

	// use 100 B as minimum size
	if size > 100 {
		new_pow -= (size as f64 / 100.).log2()
	}

	// use 100s as minimum lifetime
	if time_diff > 100 {
		new_pow -= (time_diff as f64 / 100.).log2()
	}

	return new_pow
}

fn get_current_pow(hash: &String, message: &Message) -> f64 {
	let in_dt = NaiveDateTime::parse_from_str(&message.timestamp, DT_FORMAT).unwrap();
	let time_diff = Utc::now().timestamp() - in_dt.timestamp();
	let size
		= hash.len()
		+ message.nonce.len()
		+ message.timestamp.len()
		+ message.payload.len();
	return pow_decay(get_leading_zeros(hex_to_bytes(&hash)), size, time_diff);
}

// returns hash of message with lowest remaining
// proof of work and total size of database
fn get_first_expired(db: &Arc<Database>) -> (String, usize) {
    let db = db.map.lock().unwrap();
	if db.keys().len() == 0 {
		return ("".to_string(), 0)
	}

	let mut total_size = 0;
	let mut lowest_pow = 999.0;
	let mut lowest_hash = "".to_string();
	for hash in db.keys() {
		let message = db.get(hash).unwrap();
		total_size
			+= hash.len()
			+ message.nonce.len()
			+ message.timestamp.len()
			+ message.payload.len();

		let pow = get_current_pow(hash, message) as f64;
		if lowest_pow > pow {
			lowest_pow = pow;
			lowest_hash = hash.to_string();
		}
	}
	return (lowest_hash, total_size)
}

fn handle_request(line: &str, db: &Arc<Database>, max_db_size: usize) -> String {
	// prevent timing attacks?
	// TODO: min delay and/or cap delay to 1s regardless of operation?
	thread::sleep(time::Duration::from_secs_f64(rand::random::<f64>()));

	let items: Vec<&str> = line.split_whitespace().collect();
	if items.len() == 0 {
		return "no command given".to_string()
	}

	match items[0] {
		"help" => {
			if items.len() == 1 {
				return "help, list, get, set".to_string()
			}

			match items[1] {
				"help" => return "list supported commands".to_string(),
				"list" => return "list hashes of stored messages".to_string(),
				"get" => return "get hash: return message with given hash (in hex, without leading 0x)".to_string(),
				"set" => return "set nonce timestamp payload: add given message, timestamp must be in YYYY-MM-DDThh:mm:ss format".to_string(),
				_ => return "error: no help for command '".to_string() + items[1] + "'"
			}
		},
		"list" => {
		    let db = db.map.lock().unwrap();
			if db.keys().len() == 0 {
				debug!("list: empty");
				return "empty".to_string()
			}
			let mut keys = "".to_string();
			for key in db.keys() {
				keys.push_str(&key);
				keys.push_str(" ")
			}
			debug!("list: {}", db.keys().len());
			return keys
		},
		"get" => {
			if items.len() < 2 {
				return "error: get command requires 1 argument".to_string()
			}
			trace!("get: {}", items[1]);

		    let db = db.map.lock().unwrap();
			let message = match db.get(items[1]) {
				Some(message) => message,
				None => return "no such hash".to_string()
			};

			return format!("{} {} {}", message.nonce, message.timestamp, message.payload)
		},
		"set" => {
			if items.len() < 4 {
				return "error: set command requires 3 arguments".to_string()
			}

			let message = Message{nonce: items[1].to_string(), timestamp: items[2].to_string(), payload: items[3].to_string()};

			let hash = get_hash(&message.nonce, &message.timestamp, &message.payload);
			let hash_str = bytes_to_hex(hash);
			debug!("set hash: {}", hash_str);

			let pow = get_leading_zeros(hash);
			trace!("set pow: {}", pow);
			let min_pow = 1;
			if pow < min_pow {
				return format!("error: minimum proof of work is at least {} leading zero bit(s) in SHA512 of nonce+timestamp+payload", min_pow)
			}

			if hash_str.len() + message.size() > max_db_size {
				return format!("error: message too large: {}, hash+nonce+timestamp+payload should be at most {} byte(s)", hash_str.len() + message.size(), max_db_size)
			}

			{
			    let db = db.map.lock().unwrap();
				if db.contains_key(&hash_str) {
					return "ok".to_string()
				}
			}

			// make sure timestamp isn't in future
			// prevent discovery of node's current time?
			let now = Utc::now() + Duration::milliseconds((60000. * rand::random::<f64>()) as i64);
			trace!("set local now: {}, message timestamp: {}", now, items[2]);
			let in_dt = match NaiveDateTime::parse_from_str(items[2], DT_FORMAT) {
				Ok(dt) => dt,
				Err(e) => return format!("error: invalid timestamp '{}': {}", items[2], e)
			};

			let diff = now.timestamp() - in_dt.timestamp();
			if diff < 0 {
				return "error: timestamp too far in future".to_string()
			}
			trace!("set time difference: {} s", diff);

			let (lowest_hash, total_size) = get_first_expired(&db);
		    let mut db = db.map.lock().unwrap();
			if total_size + hash_str.len() + message.size() <= max_db_size {
				debug!("total db size: {}", total_size);
				info!("set adding message: {} {} {}", &message.nonce, &message.timestamp, &message.payload);
				let _ = db.insert(hash_str, message);
				return "ok".to_string()
			} else {
				let lowest_pow = get_current_pow(&lowest_hash, &db.get(&lowest_hash).unwrap());
				trace!("lowest pow: {}, hash: {}", lowest_pow, lowest_hash);
				if (pow as f64) < lowest_pow {
					return format!("error: insufficient proof of work: {}, should be at least {}", pow, ceil(lowest_pow, 0))
				} else {
					debug!("total db size: {}", total_size);
					info!("replacing {} with {}", lowest_hash, hash_str);
					let _ = db.remove(&lowest_hash).unwrap();
					let _ = db.insert(hash_str, message);
					return "ok".to_string()
				}
			}
		},
		_ => return format!("error: unknown command '{}'", items[0])
	}
}
