1. Install rustup and cargo
```bash
curl https://sh.rustup.rs -sSf | sh
```

2. To build 
```bash
cargo build --release
```

3. To run 
```bash
./target/release/wsredis
```

fill proper environment variables to build

REDIS_ADDR (default 127.0.0.1)
REDIS_PORT (default 6379)
WS_ADDR (default 127.0.0.1)
WS_PORT (default 3030)

```bash
export VAR_NAME=VAR_VALUE
```
