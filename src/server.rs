use crate::{
    application::{Command, Deserialize, ErrorCode, Request, Response, Serialize},
    kv_store::{handler, KVStore, Key, Value},
    protocol::{MessageID, Msg, Protocol},
};
use hashlink::{LinkedHashMap, LruCache};
use std::{
    collections::{HashMap, HashSet},
    io,
    net::Ipv4Addr,
    sync::Arc,
};
use tokio::{net::UdpSocket, sync::Mutex};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Debug, Clone)]
pub struct Server {
    ip: Ipv4Addr,
    port: u16,
}

impl Server {
    #[must_use]
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Server { ip, port }
    }

    #[tracing::instrument(skip_all)]
    pub async fn _loop_body(
        &self,
        sock: Arc<UdpSocket>,
        kvstore: Arc<Mutex<KVStore>>,
        at_most_once_cache: Arc<Mutex<HashMap<MessageID, Response>>>,
        buf: &mut [u8],
    ) {
        let (len, addr) = sock.recv_from(buf).await.unwrap();
        let buf = buf[..len].to_vec();
        handler(sock, &buf, addr, kvstore, at_most_once_cache)
            .await
            .unwrap();
    }

    pub async fn run(&mut self) -> io::Result<()> {
        tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_span_events(FmtSpan::NEW)
            .with_span_events(FmtSpan::CLOSE)
            .init();

        let sock = Arc::new(UdpSocket::bind((self.ip, self.port)).await?);
        let kvstore = Arc::new(Mutex::new(KVStore::new()));
        let at_most_once_cache = Arc::new(Mutex::new(HashMap::new()));
        println!(
            "Server listening on {}:{}",
            sock.local_addr().unwrap().ip(),
            sock.local_addr().unwrap().port()
        );

        let mut buf = [0; 16 * 1024];
        loop {
            self._loop_body(
                sock.clone(),
                kvstore.clone(),
                at_most_once_cache.clone(),
                &mut buf,
            )
            .await;
        }
    }
}
