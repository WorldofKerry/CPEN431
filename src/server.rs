use crate::expiring_hashmap::ExpiringHashMap;
use crate::kv_store::{self, handle_request, KVStore};
use crate::{
    application::{Deserialize, Response, Serialize},
    protocol::{MessageID, Msg, Protocol},
};
use std::{
    io,
    net::Ipv4Addr,
    sync::Arc,
};
use get_size::GetSize;
use tokio::{net::UdpSocket, sync::Mutex};
use tracing::{info, info_span, Instrument};

type SyncKVStore = Arc<Mutex<KVStore>>;
type SyncAtMostOnceCache = Arc<Mutex<ExpiringHashMap<MessageID, Response>>>;

#[derive(Debug, Clone)]
pub struct Server {
    kvstore: SyncKVStore,
    at_most_once_cache: SyncAtMostOnceCache,
    last_time: std::time::Instant,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            kvstore: Arc::new(Mutex::new(KVStore::new())),
            at_most_once_cache: Arc::new(Mutex::new(ExpiringHashMap::new(std::time::Duration::from_secs(1)))),
            last_time: std::time::Instant::now(),
        }
    }
}

impl Server {
    // #[tracing::instrument(skip_all)]
    pub async fn _parse_bytes(
        kvstore: SyncKVStore,
        at_most_once_cache: SyncAtMostOnceCache,
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
        kvstore: SyncKVStore,
        at_most_once_cache: SyncAtMostOnceCache,
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
        let msg = match Server::_parse_bytes(kvstore, at_most_once_cache, &buf).await {
            Ok(response) => {
                response
            }
            Err(err) => {
                info!("{:?}", err);
                b"Error".to_vec()
            }
        };
        sock.send_to(&msg, addr)
            .instrument(info_span!("send_to"))
            .await?;
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
