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
use tokio::{net::UdpSocket, sync::Mutex};
use tracing::{info_span, Instrument};

#[derive(Debug, Clone)]
pub struct Server {
    kvstore: Arc<Mutex<KVStore>>,
    at_most_once_cache: Arc<Mutex<HashMap<MessageID, Response>>>,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            kvstore: Arc::new(Mutex::new(KVStore::new())),
            at_most_once_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Server {
    #[tracing::instrument(skip_all)]
    pub async fn handle_recv(
        kvstore: Arc<Mutex<KVStore>>,
        at_most_once_cache: Arc<Mutex<HashMap<[u8; 16], Response>>>,
        buf: &[u8],
    ) -> anyhow::Result<Msg> {
        let msg = Msg::from_bytes(buf)?;
        let message_id = msg.message_id();

        if let Some(response) = at_most_once_cache.lock().await.get(&message_id) {
            return Ok(response.clone().to_msg(message_id));
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
        Ok(response.to_msg(message_id))
    }

    #[tracing::instrument(skip_all)]
    pub async fn _loop_body(
        &self,
        sock: Arc<UdpSocket>,
        kvstore: Arc<Mutex<KVStore>>,
        at_most_once_cache: Arc<Mutex<HashMap<MessageID, Response>>>,
        buf: &mut [u8],
    ) -> io::Result<()> {
        let (len, addr) = sock
            .recv_from(buf)
            .instrument(info_span!("recv_from"))
            .await
            .unwrap();
        let buf = buf[..len].to_vec();
        match Server::handle_recv(kvstore, at_most_once_cache, &buf).await {
            Ok(response) => {
                let bytes = response.to_bytes();
                sock.send_to(&bytes, addr)
                    .instrument(info_span!("send_to"))
                    .await?;
            }
            Err(err) => {
                dbg!(err);
            }
        }
        Ok(())
    }

    pub async fn serve(&self, ip: Ipv4Addr, port: u16) -> io::Result<()> {
        let sock = Arc::new(UdpSocket::bind((ip, port)).await?);
        println!(
            "Server listening on {}:{}",
            sock.local_addr().unwrap().ip(),
            sock.local_addr().unwrap().port()
        );

        let mut buf = [0; 16 * 1024];
        loop {
            self._loop_body(
                sock.clone(),
                self.kvstore.clone(),
                self.at_most_once_cache.clone(),
                &mut buf,
            )
            .await?;
        }
    }
}
