# TikSync Rust SDK

**TikTok Live SDK for Rust** — Real-time chat, gifts, likes, follows & viewer events.

```rust
use tiksync::TikSync;

#[tokio::main]
async fn main() {
    let client = TikSync::new("charlidamelio", "your_api_key")
        .on("chat", |data| {
            println!("[{}] {}", data["uniqueId"], data["comment"]);
        })
        .on("gift", |data| {
            println!("{} sent {}", data["uniqueId"], data["giftName"]);
        });

    client.connect().await.unwrap();
}
```

## Installation

```toml
[dependencies]
tiksync = "1.0"
tokio = { version = "1", features = ["full"] }
```

## License

MIT — built by [SyncLive](https://synclive.fr) | [tiksync.com](https://tiksync.com)
