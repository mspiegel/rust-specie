specie: Money, especially in the form of coins made from precious metal, 
that has an intrinsic value; coinage.

specie is a collection of [cache algorithms](https://en.wikipedia.org/wiki/Cache_replacement_policies)
written in [Rust](https://www.rust-lang.org). The primary goal of this library is to write
simple implementations. Preferably with no unsafe blocks.

## Roadmap

- [x] [LRU](src/lru/mod.rs)
- [ ] ARC
- [ ] LIRS
- [ ] 2Q

## Help Needed

- Remove the extra call to self.data.get() in LRUCache::get()
- Implement get or load function for cache algorithms
