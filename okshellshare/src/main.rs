use crate::bus::bus_command_with_reply;
mod bus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let payload = std::env::var("XDPH_WINDOW_SHARING_LIST").unwrap_or_default();
    let reply: String = bus_command_with_reply("Screenshare", &payload).await?;
    println!("{reply}");
    Ok(())
}