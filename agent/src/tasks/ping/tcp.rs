use std::hint::black_box;
use std::io;
use tokio::net::TcpStream;
use tokio::time::timeout;

// TCP 系统重传时间为 1 Sec 以上，请勿动本参数
static PING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

pub async fn tcping_target(target: std::net::SocketAddr) -> io::Result<std::time::Duration> {
    let start = std::time::Instant::now();
    match timeout(PING_TIMEOUT, TcpStream::connect(target)).await {
        Ok(Ok(stream)) => {
            black_box(stream);
            Ok(start.elapsed())
        }
        _ => {
            Err(io::Error::last_os_error()) // DO NOT BELIEVE THIS
        }
    }
}
