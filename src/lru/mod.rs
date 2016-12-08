//! The 'lru' module implements a [least-recently used](
//! https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_Recently_Used_.28LRU.29) cache.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::rc::Rc;

struct CacheEntry<V> {
    // cache value
    val: V,
    // clock instant when entry was most recently accessed
    instant: u64,
}

pub struct LRUCache<K: Eq + Hash, V> {
    // maximum number of elements stored in the cache
    capacity: usize,
    // logical clock that is incremented on each operation
    clock: u64,
    // unordered map that stores (key, value) pairs
    data: HashMap<Rc<K>, CacheEntry<V>>,
    // ordered map sorted by clock instants. Used by eviction algorithm
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
            // If the (key, value) pair is located,
            // then find the logical time instant associated
            // with that key. The instant must be set to now.
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
                // If the (key, value) pair is located,
                // then delete the association with the old instant
                // and create an association to now.
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
            // If the (key, value) pair is located,
            // then replace the previous value,
            // and update the logical time association
            // with the pair.
            Entry::Occupied(mut e) => {
                let e = e.get_mut();
                let prev = e.instant;
                e.instant = now;
                e.val = val;
                self.order.remove(&prev);
                self.order.insert(now, key.clone());
                None
            }
            // If the (key, value) pair is not located,
            // then insert the new association.
            Entry::Vacant(e) => {
                let evict = {
                    if size == self.capacity {
                        // Evict the oldest entry from the clock instant map
                        let oldest = self.order.keys().cloned().next().unwrap();
                        Some(self.order.remove(&oldest).unwrap())
                    } else {
                        None
                    }
                };
                let entry = CacheEntry {
                    val: val,
                    instant: now,
                };
                e.insert(entry);
                self.order.insert(now, key.clone());
                evict
            }
        };
        match evict {
            // Evict the oldest entry from the data map
            // Moved to end of function because of borrow checker
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
