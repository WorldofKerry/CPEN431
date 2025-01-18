use std::{collections::{HashMap, HashSet}, io, net::Ipv4Addr};
use crate::{application::{Command, Deserialize, Request, Response, Serialize}, protocol::{MessageID, Msg, Protocol}};
use tokio::net::UdpSocket;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Key {
    key: Vec<u8>,
}

impl Key {
    pub fn new(key: Vec<u8>) -> Self {
        Key { key }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Value {
    value: Vec<u8>,
    version: i32,
}

impl Value {
    pub fn new(value: Vec<u8>, version: Option<i32>) -> Self {
        Value {
            value,
            version: version.unwrap_or(0),
        }
    }
    pub fn version(&self) -> i32 {
        self.version
    }
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

pub struct Server {
    ip: Ipv4Addr,
    port: u16,
    kv_data: HashMap<Key, Value>,
    at_most_once_cache: HashMap<MessageID, Response>,
}

impl Server {
    #[must_use] pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Server {
            ip,
            port,
            kv_data: HashMap::new(),
            at_most_once_cache: HashMap::new(),
        }
    }

    fn handle_request(&mut self, request: Request) -> Response {
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
                Response::success()
            },
            Request {
                command: Command::Put,
                key: Some(key),
                value: Some(value),
                version,
                ..
            } => {
                self.kv_data.insert(Key::new(key), Value::new(value, version));
                Response::success()
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
            _ => {
                panic!("Unsupported command: {:?}", request.command);
            }
        }
    }

    pub fn handle_recv(&mut self, buf: &[u8]) -> anyhow::Result<Msg> {
        let msg = Msg::from_bytes(buf)?;
        let message_id = msg.message_id();

        if self.at_most_once_cache.contains_key(&message_id) {
            println!("Received duplicate message: {:?}", message_id);
            return Ok(self.at_most_once_cache[&message_id].clone().to_msg(message_id));
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

    pub async fn run(&mut self) -> io::Result<()> {
        let sock = UdpSocket::bind((self.ip, self.port)).await?;
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
        }
    }
}
