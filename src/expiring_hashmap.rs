use std::collections::HashMap;
/// https://users.rust-lang.org/t/map-that-removes-entries-after-a-given-time-after-last-access/42767/2
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ExpiringHashMap<K, V> {
    hash_map: HashMap<K, (Instant, V)>,
    duration: Duration,
}

impl<K, V> ExpiringHashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn new(duration: Duration) -> Self {
        Self {
            hash_map: HashMap::new(),
            duration,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.hash_map.insert(k, (Instant::now(), v)).map(|v| v.1)
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        let now = Instant::now();
        let Self { hash_map, duration } = self;
        hash_map.get_mut(k).and_then(|tup| {
            if &(now - tup.0) > duration {
                None
            } else {
                tup.0 = now;
                Some(&tup.1)
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn main() {
        let mut m = ExpiringHashMap::new(Duration::from_millis(500));
        m.insert(1, 2);
        println!("{:?}", m.get(&1));
        std::thread::sleep(Duration::from_millis(400));
        println!("{:?}", m.get(&1));
        std::thread::sleep(Duration::from_millis(400));
        println!("{:?}", m.get(&1));
        std::thread::sleep(Duration::from_millis(500));
        println!("{:?}", m.get(&1));
    }
}
