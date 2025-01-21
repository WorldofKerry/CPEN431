use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use get_size::GetSize;
use tokio::sync::Mutex;

use crate::application::{Command, ErrorCode, Request, Response};

#[derive(Debug, Eq, Hash, PartialEq, GetSize, Clone)]
pub struct Key {
    pub key: Vec<u8>,
}

impl Key {
    pub fn new(key: Vec<u8>) -> Self {
        Key { key }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, GetSize, Clone)]
pub struct Value {
    pub value: Vec<u8>,
    pub version: i32,
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

#[derive(Debug)]
pub struct KVStore {
    pub data: HashMap<Key, Value>,
}

impl Default for KVStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KVStore {
    pub fn new() -> Self {
        KVStore {
            data: HashMap::new(),
        }
    }
}

impl Deref for KVStore {
    type Target = HashMap<Key, Value>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for KVStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

// #[tracing::instrument(skip_all)]
pub async fn handle_request(kvstore: Arc<Mutex<KVStore>>, request: Request) -> Response {
    match request {
        Request {
            command: Command::IsAlive,
            ..
        } => Response::success(),
        Request {
            command: Command::Wipeout,
            ..
        } => {
            kvstore.lock().await.clear();
            Response::success()
        }
        Request {
            command: Command::Put,
            key: Some(key),
            value: Some(value),
            version,
            ..
        } => {
            if key.len() > 32 {
                return Response::error(ErrorCode::InvalidKey);
            }
            if value.len() > 10000 {
                return Response::error(ErrorCode::InvalidValue);
            }
            let mut kvstore = kvstore.lock().await;
            if kvstore.get_size() > 67 * 1024 * 1024 {
                return Response::error(ErrorCode::OutOfSpace);
            }
            kvstore.insert(Key::new(key), Value::new(value, version));
            Response::success()
        }
        Request {
            command: Command::Get,
            key: Some(key),
            ..
        } => match kvstore.lock().await.get(&Key::new(key)) {
            Some(Value { value, version }) => Response {
                err_code: ErrorCode::Success,
                value: Some(value.to_vec()),
                version: Some(*version),
                ..Default::default()
            },
            None => Response::error(ErrorCode::NonExistentKey),
        },
        Request {
            command: Command::Remove,
            key: Some(key),
            ..
        } => match kvstore.lock().await.remove(&Key::new(key)) {
            Some(_) => Response::success(),
            None => Response::error(ErrorCode::NonExistentKey),
        },
        Request {
            command: Command::Shutdown,
            ..
        } => {
            std::process::exit(0);
        }
        _ => panic!("Unsupported command: {:?}", request.command),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_kvstore() {
        let kvstore = Arc::new(Mutex::new(KVStore::new()));
        let mut handles = vec![];
        for i in 0..10 {
            let inner = kvstore.clone();
            let handle = tokio::spawn(async move {
                let request = Request {
                    command: Command::Put,
                    key: Some(vec![0]),
                    value: Some(vec![i as u8]),
                    version: None,
                };
                let response = handle_request(inner, request).await;
                eprintln!("{:?}", response);
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.await.unwrap();
            eprintln!("{:?}", kvstore.lock().await);
        }
    }
}
