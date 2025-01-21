use std::collections::HashMap;


#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Key {
    pub key: Vec<u8>,
}

impl Key {
    pub fn new(key: Vec<u8>) -> Self {
        Key { key }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
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
pub struct FixedSizeKVStore {
    data: HashMap<Key, Value>,
    current_size: usize,
    max_size: usize,
}

impl FixedSizeKVStore {
    pub fn new(max_size: usize) -> Self {
        FixedSizeKVStore {
            data: HashMap::new(),
            max_size,
            current_size: std::mem::size_of::<FixedSizeKVStore>(),
        }
    }

    fn mem_size(key: &Key, value: &Value) -> usize {
        key.key.len() + value.value.len()
    }

    pub fn insert(&mut self, key: Key, value: Value) -> Result<(), ()> {
        let mem_size = Self::mem_size(&key, &value);
        if self.current_size + mem_size > self.max_size {
            return Err(());
        }
        self.data.insert(key, value);
        self.current_size += mem_size;
        Ok(())
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        if let Some(value) = self.data.remove(key) {
            self.current_size -= Self::mem_size(key, &value);
            Some(value)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.current_size = std::mem::size_of::<FixedSizeKVStore>();
    }

    pub fn approx_mem_size(&self) -> usize {
        self.current_size
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_fixed_size_kv_store() {
        let base_size = 64;
        let mut kv_store = FixedSizeKVStore::new(1000);
        assert_eq!(kv_store.approx_mem_size(), base_size);
        kv_store.insert(Key::new(vec![1, 2, 3]), Value::new(vec![4, 5], None)).unwrap();
        assert_eq!(kv_store.approx_mem_size(), base_size + 3 + 2);
    }
}