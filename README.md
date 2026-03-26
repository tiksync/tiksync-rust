<p align="center">
  <img src="https://raw.githubusercontent.com/tiksync/.github/main/profile/logo-96.png" width="60" alt="TikSync" />
</p>

<h1 align="center">TikSync Rust SDK</h1>

<p align="center">
  <strong>TikTok Live SDK for Rust</strong> — Real-time chat, gifts, likes, follows & viewer events.<br>
  <a href="https://tik-sync.com">Website</a> · <a href="https://tik-sync.com/docs">Documentation</a> · <a href="https://tik-sync.com/pricing">Pricing</a>
</p>

---

## Installation

```bash
cargo add tiksync
```

Or add to `Cargo.toml`:
```toml
[dependencies]
tiksync = "1.0"
```

## Quick Start

```rust
use tiksync::TikSync;

#[tokio::main]
async fn main() {
    let mut client = TikSync::new("username", "your_api_key");

    client.on_chat(|data| {
        println!("[{}] {}", data.unique_id, data.comment);
    });

    client.on_gift(|data| {
        println!("{} sent {} x{}", data.unique_id, data.gift_name, data.repeat_count);
    });

    client.connect().await.unwrap();
}
```

Requires Rust 1.75+. Async runtime: Tokio.

## Events

| Event | Description |
|-------|-------------|
| `connected` | Connected to stream |
| `chat` | Chat message received |
| `gift` | Gift received (with diamond count, streak info) |
| `like` | Likes received |
| `follow` | New follower |
| `share` | Stream shared |
| `member` | User joined the stream |
| `roomUser` | Viewer count update |
| `streamEnd` | Stream ended |
| `disconnected` | Disconnected |
| `error` | Connection error |

## Get Your API Key

1. Sign up at [tik-sync.com](https://tik-sync.com)
2. Go to Dashboard → API Keys
3. Create a new key

Free tier: 1,000 requests/day, 10 WebSocket connections.

## License

MIT — Built by [TikSync](https://tik-sync.com)
