use crate::kv_store::handle_request;
use crate::{
    application::{Deserialize, Response, Serialize},
    kv_store::KVStore,
    protocol::{MessageID, Msg, Protocol},
};
use std::{
    collections::HashMap,
    io,
    net::Ipv4Addr,
    sync::Arc,
};
use get_size::GetSize;
use tokio::{net::UdpSocket, sync::Mutex};
use tracing::{info_span, Instrument};

#[derive(Debug, Clone)]
pub struct Server {
    kvstore: Arc<Mutex<KVStore>>,
    at_most_once_cache: Arc<Mutex<HashMap<MessageID, Response>>>,
    last_time: std::time::Instant,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            kvstore: Arc::new(Mutex::new(KVStore::new())),
            at_most_once_cache: Arc::new(Mutex::new(HashMap::new())),
            last_time: std::time::Instant::now(),
        }
    }
}

impl Server {
    // #[tracing::instrument(skip_all)]
    pub async fn _parse_bytes(
        kvstore: Arc<Mutex<KVStore>>,
        at_most_once_cache: Arc<Mutex<HashMap<[u8; 16], Response>>>,
        buf: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let msg = Msg::from_bytes(buf)?;
        let message_id = msg.message_id();

        if let Some(response) = at_most_once_cache.lock().await.get(&message_id) {
            return Ok(response.clone().to_msg(message_id).to_bytes());
        }

        let response = match msg.payload() {
            Ok(request) => {
                handle_request(kvstore, request.clone()).await
            }
            Err(err) => {
                dbg!(&err);
                Response::error(err.into())
            }
        };
        at_most_once_cache
            .lock()
            .await
            .insert(message_id, response.clone());
        Ok(response.to_msg(message_id).to_bytes())
    }

    // #[tracing::instrument(skip_all)]
    pub async fn _listen_socket(
        &mut self,
        sock: Arc<UdpSocket>,
        kvstore: Arc<Mutex<KVStore>>,
        at_most_once_cache: Arc<Mutex<HashMap<MessageID, Response>>>,
        buf: &mut [u8],
    ) -> io::Result<()> {

        if self.last_time.elapsed().as_millis() >= 250 {
            self.last_time = std::time::Instant::now();
            let pid = std::process::id();
            let output = tokio::process::Command::new("ps")
                .arg("-o")
                .arg("rss=")
                .arg(format!("{}", pid))
                .output()
                .await
                .unwrap();
            let memory_usage = String::from_utf8(output.stdout).unwrap().trim().parse::<f64>().unwrap() / 1024.0;
            println!("Memory Usage: {:.2} MB", memory_usage);
            println!("kvstore size: {}", kvstore.lock().await.get_size());
        }

        let (len, addr) = sock
            .recv_from(buf)
            .instrument(info_span!("recv_from"))
            .await
            .unwrap();
        let buf = buf[..len].to_vec();
        match Server::_parse_bytes(kvstore, at_most_once_cache, &buf).await {
            Ok(response) => {
                sock.send_to(&response, addr)
                    .instrument(info_span!("send_to"))
                    .await?;
            }
            Err(err) => {
                dbg!(err);
            }
        }
        Ok(())
    }

    pub async fn serve(&mut self, ip: Ipv4Addr, port: u16) -> io::Result<()> {
        let sock = Arc::new(UdpSocket::bind((ip, port)).await?);
        println!(
            "Server listening on {}:{}",
            sock.local_addr().unwrap().ip(),
            sock.local_addr().unwrap().port()
        );

        let mut buf = [0; 16 * 1024];
        loop {
            self._listen_socket(
                sock.clone(),
                self.kvstore.clone(),
                self.at_most_once_cache.clone(),
                &mut buf,
            )
            .await?;
        }
    }
}
