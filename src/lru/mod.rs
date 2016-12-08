use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::rc::Rc;

struct CacheEntry<V> {
    val: V,
    instant: u64,
}

pub struct LRUCache<K: Eq + Hash, V> {
    capacity: usize,
    clock: u64,
    data: HashMap<Rc<K>, CacheEntry<V>>,
    order: BTreeMap<u64, Rc<K>>,
}

impl<K, V> LRUCache<K, V>
    where K: Eq + Hash
{
    pub fn new(capacity: usize) -> LRUCache<K, V> {
        LRUCache {
            capacity: capacity,
            clock: 0,
            data: HashMap::with_capacity(capacity),
            order: BTreeMap::new(),
        }
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        let now = self.clock;
        let key = Rc::new(key);
        let prev = match self.data.entry(key.clone()) {
            Entry::Occupied(mut e) => {
                self.clock += 1;
                let e = e.get_mut();
                let prev = e.instant;
                e.instant = now;
                Some(prev)
            }
            _ => None,
        };
        match prev {
            Some(t) => {
                let k = self.order.remove(&t);
                self.order.insert(now, k.unwrap());
                self.data.get(key.as_ref()).as_ref().map(|x| &x.val)
            }
            None => None,
        }
    }

    pub fn insert(&mut self, key: K, val: V) {
        let now = self.clock;
        self.clock += 1;
        let size = self.data.len();
        let key = Rc::new(key);
        let evict = match self.data.entry(key.clone()) {
            Entry::Occupied(mut e) => {
                let e = e.get_mut();
                let prev = e.instant;
                e.instant = now;
                e.val = val;
                self.order.remove(&prev);
                self.order.insert(now, key.clone());
                None
            }
            Entry::Vacant(e) => {
                let entry = CacheEntry {
                    val: val,
                    instant: now,
                };
                e.insert(entry);
                self.order.insert(now, key.clone());
                if size == self.capacity {
                    let oldest = self.order.keys().cloned().next().unwrap();
                    Some(self.order.remove(&oldest).unwrap())
                } else {
                    None
                }
            }
        };
        match evict {
            Some(k) => {
                self.data.remove(k.as_ref());
            }
            None => {}
        };
    }

    pub fn len(&self) -> usize {
        debug_assert!(self.data.len() == self.order.len());
        self.data.len()
    }
}

#[test]
fn lru_cache() {
    let mut cache = LRUCache::new(3);
    assert_eq!(0, cache.clock);
    assert_eq!(0, cache.len());

    cache.insert(1, 2);
    cache.insert(3, 4);
    cache.insert(5, 6);
    assert_eq!(3, cache.clock);
    assert_eq!(3, cache.len());
    assert_eq!(Some(&2), cache.get(1));
    assert_eq!(Some(&4), cache.get(3));
    assert_eq!(Some(&6), cache.get(5));
    assert_eq!(None, cache.get(7));
    assert_eq!(6, cache.clock);

    cache.insert(1, 1);
    cache.insert(3, 3);
    cache.insert(5, 6);
    assert_eq!(9, cache.clock);
    assert_eq!(3, cache.len());
    assert_eq!(Some(&1), cache.get(1));
    assert_eq!(Some(&3), cache.get(3));
    assert_eq!(Some(&6), cache.get(5));
    assert_eq!(None, cache.get(7));

    cache.insert(7, 8);
    assert_eq!(3, cache.len());
    assert_eq!(None, cache.get(1));
    assert_eq!(Some(&3), cache.get(3));
    assert_eq!(Some(&6), cache.get(5));
    assert_eq!(Some(&8), cache.get(7));
    assert_eq!(16, cache.clock);
}
