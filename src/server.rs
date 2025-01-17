use std::{collections::HashMap, io, net::Ipv4Addr};
use crate::{application::{Command, Deserialize, Request, Response, Serialize}, protocol::{Msg, Protocol}};
use tokio::net::UdpSocket;

pub struct Server {
    ip: Ipv4Addr,
    port: u16,
    kv_data: HashMap<Vec<u8>, Vec<u8>>,
}

impl Server {
    #[must_use] pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Server {
            ip,
            port,
            kv_data: HashMap::new(),
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
                ..
            } => {
                self.kv_data.insert(key, value);
                Response::success()
            },
            Request {
                command: Command::Get,
                key: Some(key),
                ..
            } => {
                let value = self.kv_data.get(&key).cloned();
                Response {
                    err_code: if value.is_some() { 0 } else { 1 },
                    value,
                    ..Default::default()
                }
            },
            Request {
                command: Command::Remove,
                key: Some(key),
                ..
            } => {
                let value = self.kv_data.remove(&key);
                Response {
                    err_code: if value.is_some() { 0 } else { 1 },
                    value,
                    ..Default::default()
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

        match msg.payload() {
            Ok(request) => {
                println!("Received request: {request:?}");
                let response = self.handle_request(request);
                Ok(response.to_msg(message_id))
            }
            Err(e) => {
                Ok(Response::error(e.into()).to_msg(message_id))
            }
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let sock = UdpSocket::bind((self.ip, self.port)).await?;
        println!("Server listening on {}:{}", sock.local_addr().unwrap().ip(), sock.local_addr().unwrap().port());
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = sock.recv_from(&mut buf).await?;

            match self.handle_recv(&buf[..len]) {
                Ok(response) => {
                    sock.send_to(&response.to_bytes(), addr).await?;
                }
                Err(e) => {
                    eprintln!("Error: {e:?}");
                }
            }
        }
    }
}
