use std::{collections::{HashMap, HashSet}, io, net::Ipv4Addr};
use crate::{application::{Command, Deserialize, Request, Response, Serialize}, kv_store::{Key, Value}, protocol::{MessageID, Msg, Protocol}};
use hashlink::{LinkedHashMap, LruCache};
use tokio::net::UdpSocket;

#[derive(Debug)]
struct Metrics {
    requests: HashMap<Command, u64>,
}

pub struct Server {
    ip: Ipv4Addr,
    port: u16,
    kv_data: HashMap<Key, Value>,
    at_most_once_cache: LruCache<MessageID, Response>,
    metrics: Metrics,
}

impl Server {
    #[must_use] pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Server {
            ip,
            port,
            kv_data: HashMap::new(),
            at_most_once_cache: LruCache::new(100),
            metrics: Metrics {
                requests: HashMap::new(),
            },
        }
    }

    #[must_use] fn get_kv_size(&self) -> usize {
        // TODO: make this a size counter that increments/decrements as entries added/removed
        self.kv_data.iter().fold(0, |acc, (k, v)| acc + k.key.len() + v.value.len())
    }

    fn handle_request(&mut self, request: Request) -> Response {
        self.metrics.requests.entry(request.command.clone()).and_modify(|v| *v += 1).or_insert(1);
        match request {
            Request {
                command: Command::IsAlive,
                ..
            } => Response::success(),
            Request {
                command: Command::Wipeout,
                ..
            } => {
                self.kv_data.clear();
                self.at_most_once_cache.clear();
                Response::success()
            },
            Request {
                command: Command::Put,
                key: Some(key),
                value: Some(value),
                version,
                ..
            } => {
                if key.len() > 32 {
                    Response::error(crate::application::ErrorCode::InvalidKey)
                } else if value.len() > 10000 {
                    Response::error(crate::application::ErrorCode::InvalidValue)
                } else if self.get_kv_size() > 60 * 1024 * 1024 {
                    Response::error(crate::application::ErrorCode::OutOfSpace)
                } else {
                    self.kv_data.insert(Key::new(key), Value::new(value, version));
                    Response::success()
                }
            },
            Request {
                command: Command::Get,
                key: Some(key),
                ..
            } => {
                let value = self.kv_data.get(&Key::new(key));
                match value {
                    Some(value) => Response {
                        value: Some(value.value().to_vec()),
                        version: Some(value.version()),
                        ..Default::default()
                    },
                    None => Response::error(crate::application::ErrorCode::NonExistentKey),
                }
            },
            Request {
                command: Command::Remove,
                key: Some(key),
                ..
            } => {
                let value = self.kv_data.remove(&Key::new(key));
                match value {
                    Some(_) => Response::success(),
                    None => Response::error(crate::application::ErrorCode::NonExistentKey),
                }
            }
            Request {
                command: Command::Shutdown,
                ..
            } => {
                std::process::exit(0);
            },
            _ => {
                panic!("Unsupported command: {:?}", request.command);
            }
        }
    }

    pub fn handle_recv(&mut self, buf: &[u8]) -> anyhow::Result<Msg> {
        let msg = Msg::from_bytes(buf)?;
        let message_id = msg.message_id();

        if let Some(response) = self.at_most_once_cache.get(&message_id) {
            return Ok(response.clone().to_msg(message_id));
        }

        let response = match msg.payload() {
            Ok(request) => {
                let response = self.handle_request(request);
                response
            }
            Err(e) => {
                Response::error(e.into())
            }
        };
        self.at_most_once_cache.insert(message_id, response.clone());
        Ok(response.to_msg(message_id))
    }

    async fn log(&self) {
        println!("KV Data: {:?}", self.kv_data.len());
        println!("KV Data Size: {:?}", self.get_kv_size());
        println!("At Most Once Cache: {:?}", self.at_most_once_cache.len());
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
        println!("Metrics: {:?}", self.metrics);
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let sock = UdpSocket::bind((self.ip, self.port)).await?;
        let mut last_log_time = tokio::time::Instant::now();
        println!("Server listening on {}:{}", sock.local_addr().unwrap().ip(), sock.local_addr().unwrap().port());
        let mut buf = [0; 16 * 1024];
        loop {
            let (len, addr) = sock.recv_from(&mut buf).await?;

            match self.handle_recv(&buf[..len]) {
                Ok(response) => {
                    sock.send_to(&response.to_bytes(), addr).await?;
                }
                Err(err) => {
                    dbg!(err);
                }
            }

            if tokio::time::Instant::now().duration_since(last_log_time).as_millis() >= 1000 {
                self.log().await;
                last_log_time = tokio::time::Instant::now();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_kv_size() {
        let mut server = Server::new(Ipv4Addr::LOCALHOST, 0);
        let key = vec![1, 2, 3, 4, 5];
        let value = vec![6, 7, 8, 9, 10, 11];
        server.handle_request(Request {
            command: Command::Put,
            key: Some(key.clone()),
            value: Some(value.clone()),
            version: None,
        });
        assert_eq!(server.get_kv_size(), key.len() + value.len());
    }
}