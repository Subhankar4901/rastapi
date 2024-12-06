use super::{dlink_list::{Dlinklist, Node},FileData};
use std::{collections::HashMap, hash::Hash, ptr::NonNull, time::SystemTime};

/// Value stored inside a cache entry.
pub(super) struct CacheValue<K> {
    val: NonNull<Vec<u8>>,
    last_updated: SystemTime,
    freq_cnt: usize,
    ptr_to_list: Option<NonNull<Node<K>>>,
}
impl<K> CacheValue<K> {
    pub fn new() -> Self {
        //SAFETY : Box doesn't give NULL pointer.
        Self {
            val: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Vec::new()))) },
            last_updated: SystemTime::now(),
            freq_cnt: 1,
            ptr_to_list: None,
        }
    }
}
unsafe impl<T> Send for CacheValue<T> {}
unsafe impl<T> Sync for CacheValue<T> {}
/// Cache Store. It's a individual LFU-LRU cache.
pub(super) struct CacheStore<K: Eq + PartialEq + Hash + Clone> {
    pub(super) cache_map: HashMap<K, CacheValue<K>>,
    pub(super) lfu_list_map: HashMap<usize, Dlinklist<K>>,
    pub(super) lfu_cnt: usize,
    capacity: usize,
    pub(super) cur_size: usize,
}
impl<K: Eq + PartialEq + Hash + Clone> CacheStore<K> {
    pub fn new(capacity_in_mb: usize) -> Self {
        Self {
            cache_map: HashMap::new(),
            lfu_list_map: HashMap::new(),
            lfu_cnt: 1,
            capacity: capacity_in_mb * 1024 * 1024,
            cur_size: 0,
        }
    }
    pub fn touch(&mut self, key: K) {
        let cache_value_opt = self.cache_map.get_mut(&key);
        if let Some(cache_value) = cache_value_opt {
            if let Some(cur_lfu_list) = self.lfu_list_map.get_mut(&cache_value.freq_cnt) {
                cur_lfu_list.erase(cache_value.ptr_to_list.unwrap());
                if cur_lfu_list.size == 0 && self.lfu_cnt == cache_value.freq_cnt {
                    self.lfu_cnt += 1;
                }
            };
            cache_value.freq_cnt += 1;
            if let Some(new_lfu_list) = self.lfu_list_map.get_mut(&cache_value.freq_cnt) {
                new_lfu_list.push_front(key);
                cache_value.ptr_to_list = new_lfu_list.head;
            } else {
                let mut new_lfu_list = Dlinklist::<K>::new();
                new_lfu_list.push_front(key);
                cache_value.ptr_to_list = new_lfu_list.head;
                self.lfu_list_map.insert(cache_value.freq_cnt, new_lfu_list);
            }
        }
    }
    pub fn get(&mut self, key: K) -> Option<FileData> {
        let mut res = None;
        if let Some(cache_value) = self.cache_map.get(&key) {
            // SAFETY : cache_value.val doesn't point to a NULL pointer cuz I have initialized it with a empty vector.
            res = unsafe {
                Some(FileData::new(
                    cache_value.val.as_ref(),
                    cache_value.last_updated,
                ))
            };
        };
        if res.is_some() {
            self.touch(key);
        }
        return res;
    }
    pub fn evict_key(&mut self, key: K) {
        if let Some(cache_value) = self.cache_map.get(&key) {
            if let Some(lfu_list) = self.lfu_list_map.get_mut(&cache_value.freq_cnt) {
                lfu_list.erase(cache_value.ptr_to_list.unwrap());
            }
            //SAFETY : cache_value.val doesn't point to a NULL pointer cuz I have initialized it with a empty vector.
            let _ = unsafe { Box::from_raw(cache_value.val.as_ptr()) };
            self.cur_size = self.cur_size - unsafe { cache_value.val.as_ref().len() }
        }
        self.cache_map.remove(&key);
        if self.cache_map.len() > 0 {
            while let Some(cur_list) = self.lfu_list_map.get(&self.lfu_cnt) {
                if cur_list.size > 0 {
                    break;
                }
                self.lfu_cnt += 1;
            }
        }
    }
    pub fn evict(&mut self) {
        if let Some(lfu_list) = self.lfu_list_map.get(&self.lfu_cnt) {
            let back = lfu_list.back_clone().unwrap();
            self.evict_key(back);
        }
    }
    pub fn insert(&mut self, key: K, value: &Vec<u8>) -> usize {
        if value.len() > self.capacity || value.is_empty() {
            return self.cur_size;
        }
        let mut old_freq_cnt: usize = 0;
        if let Some(old_cache_value) = self.cache_map.get(&key) {
            old_freq_cnt = old_cache_value.freq_cnt;
            self.evict_key(key.clone());
        }
        while self.cur_size + value.len() > self.capacity {
            self.evict();
        }
        let mut new_cache_value = CacheValue::<K>::new();
        new_cache_value.freq_cnt = new_cache_value.freq_cnt.max(old_freq_cnt);
        //SAFETY : new_cache_value.val doesn't point to a NULL pointer cuz I have initialized it with a empty vector.
        unsafe { new_cache_value.val.as_mut() }.extend_from_slice(value);
        if let Some(lfu_list) = self.lfu_list_map.get_mut(&new_cache_value.freq_cnt) {
            lfu_list.push_front(key.clone());
            new_cache_value.ptr_to_list = lfu_list.head;
        } else {
            let mut new_lfu_list = Dlinklist::<K>::new();
            new_lfu_list.push_front(key.clone());
            new_cache_value.ptr_to_list = new_lfu_list.head;
            self.lfu_list_map
                .insert(new_cache_value.freq_cnt, new_lfu_list);
        }
        self.lfu_cnt = self.lfu_cnt.min(new_cache_value.freq_cnt);
        self.cache_map.insert(key, new_cache_value);
        self.cur_size += value.len();
        return self.cur_size;
    }
}
impl<K: Eq + PartialEq + Hash + Clone> Drop for CacheStore<K> {
    fn drop(&mut self) {
        for cache_val in self.cache_map.values_mut() {
            cache_val.ptr_to_list = None;
            //SAFETY : cache_value.val doesn't point to a NULL pointer cuz I have initialized it with a empty vector.
            let _ = unsafe { Box::from_raw(cache_val.val.as_ptr()) };
        }
    }
}
