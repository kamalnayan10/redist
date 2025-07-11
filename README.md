# redist

A small Redis-like server written in **Rust** using **Tokio** for async networking.  
This project demonstrates my skills in building scalable networked applications in Rust, handling multiple concurrent clients, working with shared state, and manually parsing the RESP protocol.

---

## ✅ Features

- `PING` → Responds with `PONG`
- `ECHO <value>` → Returns the same value
- `SET <key> <value>` → Stores the key-value pair
- `SET <key> <value> PX <milliseconds>` → Stores the key-value pair with an expiry time
- `GET <key>` → Returns the value or `nil` if expired or not found
- Handles multiple clients concurrently
- In-memory key-value store with optional TTL

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Redis CLI](https://redis.io/docs/getting-started/installation/) (optional, for testing)

### Run the server

```bash
git clone https://github.com/<your-username>/redist.git
cd redist
cargo run
```

The server listens on `127.0.0.1:6379` by default.

---

## 💻 Example Usage

With `redis-cli`:

```bash
redis-cli -p 6379
> PING
PONG

> SET mykey hello
OK

> GET mykey
"hello"

> SET tempkey world PX 1000
OK

# wait 1 second
> GET tempkey
(nil)
```

---

## 🛠 Tech Stack

- **Rust** for performance and safety
- **Tokio** for async networking
- `Arc<Mutex<_>>` for shared state
- Manual RESP protocol parsing

---

## 💡 Why This Exists

This project helped me understand how a basic Redis server works under the hood.  
It shows that I know how to use **Tokio** for writing async TCP servers, manage concurrency using shared state, and implement basic protocol handling.  
I’ll keep adding more features and improvements over time.

---

## 📈 What’s Next

- Add support for more Redis commands (`DEL`, `MGET`, `MSET`, etc.)
- Add background task for cleaning up expired keys
- Add basic persistence (e.g., RDB snapshotting)
- Use `dashmap` or other lock-free structures for better performance

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
