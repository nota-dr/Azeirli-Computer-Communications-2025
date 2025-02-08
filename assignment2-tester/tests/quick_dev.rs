use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn quick_dev() -> Result<()> {
    println!();
    let addr = "127.0.0.1:8080";
    let mut stream = tokio::net::TcpStream::connect(addr).await?;
    let path = "/absolute";
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        path
    );
    stream.write_all(req.as_bytes()).await?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    println!("{}", String::from_utf8_lossy(&buf));
    println!();
    Ok(())
}
