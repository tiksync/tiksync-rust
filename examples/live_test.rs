use tiksync::TikSync;

#[tokio::main]
async fn main() {
    let client = TikSync::new("letoile_de_melya", "sl_int_synclive_electron_prod_2026")
        .with_url("https://api.synclive.fr")
        .on("connected", |_| println!("[Rust] CONNECTED"))
        .on("roomInfo", |d| {
            println!("[Rust] roomInfo: {:?}", d.get("roomId"));
        })
        .on("chat", |d| {
            println!(
                "[Rust] chat: @{}",
                d.get("uniqueId").map(|v| v.to_string()).unwrap_or_default()
            );
        })
        .on("gift", |d| {
            println!(
                "[Rust] gift: @{} sent {}",
                d.get("uniqueId").map(|v| v.to_string()).unwrap_or_default(),
                d.get("giftName").map(|v| v.to_string()).unwrap_or_default()
            );
        })
        .on("like", |_| println!("[Rust] like"))
        .on("roomUser", |_| println!("[Rust] roomUser"))
        .on("error", |d| println!("[Rust] ERROR: {:?}", d))
        .on("disconnected", |_| println!("[Rust] disconnected"));

    println!("Connecting to letoile_de_melya...");
    if let Err(e) = client.connect().await {
        println!("[Rust] Connection error: {}", e);
    }
}
