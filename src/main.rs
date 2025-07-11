#![allow(unused_imports)]
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io::{Read, Write};
use std::io::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Instant, Duration};

type DB = Arc<Mutex<HashMap<String, (String,Option<Instant>)>>>;

fn parse_resp(message_bytes: &[u8]) -> Option<Vec<String>> {
    let mut idx = 0;

    if message_bytes[idx] != b'*' {
        return None;
    }
    idx += 1;

    let mut end_idx = idx;
    while message_bytes[end_idx] != b'\r' {
        end_idx += 1;
    }

    let count = std::str::from_utf8(&message_bytes[idx..end_idx]).ok()?.parse::<usize>().ok()?;
    idx = end_idx + 2; // skip \r\n

    let mut result = Vec::new();

    for _ in 0..count {
        if message_bytes[idx] != b'$' {
            return None;
        }
        idx += 1;

        let mut len_end = idx;
        while message_bytes[len_end] != b'\r' {
            len_end += 1;
        }

        let len = std::str::from_utf8(&message_bytes[idx..len_end]).ok()?.parse::<usize>().ok()?;
        idx = len_end + 2; // skip \r\n

        if idx + len > message_bytes.len() {
            return None;
        }

        let val = std::str::from_utf8(&message_bytes[idx..idx + len]).ok()?.to_string();
        result.push(val);
        idx += len + 2; // Skip past the value and \r\n
    }

    Some(result)
}


async fn handle_client(mut stream: TcpStream, db:Arc<Mutex<HashMap<String, (String,Option<Instant>)>>>) {
    let mut buf = [0; 512];

    loop {
        let read_count = match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        let args = match parse_resp(&buf[..read_count]) {
            Some(a) => a,
            None => continue, // ignore invalid RESP
        };
        
        // parse ECHO cmd
        if args.get(0).map(|s| s.eq_ignore_ascii_case("ECHO")).unwrap_or(false) && args.len() >= 2 {
            
            let value = &args[1];
            let reply = format!("${}\r\n{}\r\n", value.len(), value);
            if let Err(e) = stream.write_all(reply.as_bytes()).await {
                eprintln!("Write error: {}", e);
                break;
            }
        } // parse SET cmd
        else if args.get(0).map(|s| s.eq_ignore_ascii_case("SET")).unwrap_or(false) && args.len() >= 2 {
            
            let key = &args[1];
            let val = &args[2];
            let mut time_arg = None;
            let mut ttl = None;

            if args.len() > 4 {
                time_arg = Some(&args[3]);
                ttl = Some(&args[4]);
            }

            
            let mut store = db.lock().await;
            if !time_arg.is_none() && time_arg.map(|c| c.eq_ignore_ascii_case("PX")).unwrap_or(false){
                
                let ttl_ms = ttl.unwrap().parse::<u64>().unwrap();
                let current_time = Instant::now();
                let expires_at = Some(current_time + Duration::from_millis(ttl_ms));

                store.insert(key.clone(), (val.clone(), expires_at));

            }
            else{
                store.insert(key.clone(), (val.clone(), None)); // insert cloned strings into the HashMap
            }
            

            let reply = format!("+OK\r\n");
            if let Err(e) = stream.write_all(reply.as_bytes()).await {
                eprintln!("Write error: {}", e);
                break;
            }
        } // parse GET cmd
        else if args.get(0).map(|s| s.eq_ignore_ascii_case("GET")).unwrap_or(false) && args.len() >= 2 {
            
            let key = &args[1];
            let mut store = db.lock().await;
            
            let current_time = Instant::now();

            let response = match store.get(key) {
                Some((_val, Some(expiry))) if current_time >= *expiry => {
                    
                    store.remove(key);
                    "$-1\r\n".to_string()
                },
                Some((val, _)) => format!("${}\r\n{}\r\n", val.len(), val),
                None => "$-1\r\n".to_string()
            };

            if let Err(e) = stream.write_all(response.as_bytes()).await {
                eprintln!("Write error: {}", e);
                break;
            }
                        
        } // invalid cmd
        else {
            if let Err(e) = stream.write_all(b"+PONG\r\n").await {
                eprintln!("Write error: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db: DB = Arc::new(Mutex::new(HashMap::new()));
    
    loop{
        match listener.accept().await {
            
            Ok((stream, _addr)) => {
                let db: Arc<Mutex<HashMap<String, (String,Option<Instant>)>>> = db.clone();

                tokio::spawn(async move {
                    handle_client(stream, db).await;
                });
            },
            Err(e) => {
                eprintln!("Accept error: {}", e);
            }
        }
    }
}
