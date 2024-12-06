#![allow(dead_code)]
use std::io;
use std::sync::Mutex;
use std::{collections::HashMap, hash::Hash,time::SystemTime};
use cache_store::CacheStore;

mod cache_store;
mod dlink_list;
///Value to return when quaried against a key that exists.
pub struct FileData {
    pub val: Vec<u8>,
    pub last_updated: SystemTime,
}
impl FileData {
    pub fn new(val: &Vec<u8>, last_updated: SystemTime) -> Self {
        Self {
            val: val.clone(),
            last_updated,
        }
    }
}
///Cache metadata that contains cur index of a CacheStore in which we should insert & Current size of all CacheStores.
struct CacheMetaData {
    size_map: Vec<usize>,
    cur_idx: usize,
}
impl CacheMetaData {
    pub fn new(directory_len: usize) -> Self {
        Self {
            size_map: vec![0_usize; directory_len as usize],
            cur_idx: 0,
        }
    }
}
///The main Cache. It holds multiple CacheStores in a vector.
pub struct Cache<K: Eq + Hash + PartialEq + Clone> {
    cache_directory: Vec<Mutex<CacheStore<K>>>,
    key_cache_map: Mutex<HashMap<K, usize>>,
    cache_metadat: Mutex<CacheMetaData>,
    each_cache_size: usize,
}
impl<K: Eq + Hash + PartialEq + Clone> Cache<K> {
    pub fn new(directory_len: usize, each_cache_size: usize) -> Self {
        let mut cache_dir = Vec::<Mutex<CacheStore<K>>>::with_capacity(directory_len);
        for _ in 0..directory_len {
            cache_dir.push(Mutex::new(CacheStore::<K>::new(each_cache_size)));
        }
        let mut size_map = Vec::<usize>::with_capacity(directory_len);
        for _ in 0..directory_len {
            size_map.push(0);
        }

        Self {
            cache_directory: cache_dir,
            key_cache_map: Mutex::new(HashMap::new()),
            cache_metadat: Mutex::new(CacheMetaData::new(directory_len)),
            each_cache_size,
        }
    }
    fn find_insertion_idx(&self, size: usize) -> usize {
        let mut metadata = self.cache_metadat.lock().unwrap();
        let mut temp_idx = metadata.cur_idx;
        let mut found = false;
        loop {
            if metadata.size_map[temp_idx as usize] + size <= self.each_cache_size {
                found = true;
                break;
            }
            if (temp_idx + 1) % metadata.size_map.len() == metadata.cur_idx {
                break;
            }
            temp_idx = (temp_idx + 1) % metadata.size_map.len();
        }
        if found {
            metadata.cur_idx = (temp_idx + 1) % metadata.size_map.len();
            return temp_idx;
        } else {
            metadata.cur_idx = (metadata.cur_idx + 1) % metadata.size_map.len();
            return (temp_idx + 1) % metadata.size_map.len();
        }
    }
    pub fn insert(&self, key: K, value: &Vec<u8>) {
        if value.is_empty() {
            return;
        }
        let idx = self.find_insertion_idx(value.len());
        {
            let mut key_map_lock = self.key_cache_map.lock().unwrap();
            key_map_lock.insert(key.clone(), idx);
        }
        let mut cache_store_lock = self.cache_directory[idx].lock().unwrap();
        let cur_size = cache_store_lock.insert(key, value);
        let mut metadata_lock = self.cache_metadat.lock().unwrap();
        metadata_lock.size_map[idx as usize] = cur_size;
    }
    pub fn get(&self, key: K) -> Option<FileData> {
        let idx_opt = {
            let key_map_lock = self.key_cache_map.lock().unwrap();
            key_map_lock.get(&key).map(|t| *t)
        };
        if let Some(idx) = idx_opt {
            let mut cache_store_lock = self.cache_directory[idx].lock().unwrap();
            let res = cache_store_lock.get(key.clone());
            if res.is_none() {
                let mut key_map_lock = self.key_cache_map.lock().unwrap();
                key_map_lock.remove(&key);
            }
            return res;
        }
        return None;
    }
    pub fn try_insert(&self, key: K, value: &Vec<u8>) -> Result<(), io::Error> {
        if value.is_empty() {
            return Err(io::ErrorKind::WriteZero.into());
        }
        let idx = self.find_insertion_idx(value.len());
        {
            let mut key_map_lock = self.key_cache_map.lock().unwrap();
            key_map_lock.insert(key.clone(), idx);
        }
        let mut cache_store_lock = match self.cache_directory[idx].try_lock() {
            Ok(gurd) => gurd,
            Err(_e) => {
                return Err(io::ErrorKind::WouldBlock.into());
            }
        };
        let cur_size = cache_store_lock.insert(key, value);
        let mut metadata_lock = self.cache_metadat.lock().unwrap();
        metadata_lock.size_map[idx as usize] = cur_size;
        Ok(())
    }
    pub fn try_get(&self, key: K) -> Result<Option<FileData>, io::Error> {
        let idx_opt = {
            let key_map_lock = self.key_cache_map.lock().unwrap();
            key_map_lock.get(&key).map(|t| *t)
        };
        if let Some(idx) = idx_opt {
            let mut cache_store_lock = match self.cache_directory[idx].try_lock() {
                Ok(gurd) => gurd,
                Err(_e) => {
                    return Err(io::ErrorKind::WouldBlock.into());
                }
            };
            let res = cache_store_lock.get(key.clone());
            drop(cache_store_lock);
            if res.is_none() {
                let mut key_map_lock = self.key_cache_map.lock().unwrap();
                key_map_lock.remove(&key);
            }
            return Ok(res);
        }
        return Ok(None);
    }
    pub fn size(&self) -> usize {
        let mut sz: usize = 0;
        let metadata = self.cache_metadat.lock().unwrap();
        for i in metadata.size_map.iter() {
            sz += *i;
        }
        sz
    }
}
impl<K: Eq + Hash + PartialEq + Clone> Drop for Cache<K> {
    fn drop(&mut self) {
        for cache_store in self.cache_directory.iter_mut() {
            drop(cache_store.lock().unwrap());
        }
        self.key_cache_map = Mutex::new(HashMap::<K, usize>::new());
        self.cache_metadat = Mutex::new(CacheMetaData::new(0));
    }
}
#[cfg(test)]
mod cachetest {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn cache_size_test() {
        let lfu_cache = Cache::<String>::new(5, 1);
        let data = "abcdefghijklmnopqrstwxyzABCDEFGHIJKLMNOPQRSTUWXYZ0123456789()!@#$%^&&*()"
            .as_bytes()
            .to_vec(); //size : 72B;
        for i in 0..10000 {
            lfu_cache.insert(format!("files/img/i{i}.png"), &data);
        }
        let mut cur_total_size = data.len() * 10000; //size : 720000 bytes = 703.125 KB
        assert!(cur_total_size == lfu_cache.size(), "CACHE SIZE < CAPACITY");
        for i in 0..70000 {
            let j = i + 10000;
            lfu_cache.insert(format!("files/img/i{j}"), &data);
        }
        cur_total_size += data.len() * 70000;
        assert!(cur_total_size > lfu_cache.size(), "CACHE SIZE CAPACITY HIT");
        assert!(
            lfu_cache.size() == 5242680_usize,
            "MAXIMUM CACHE SIZE POSSIBLE"
        ); //5*floor(1024*1024/72)*72 = 5242680
    }
    #[test]
    fn cache_integrity_test() {
        let lfu_cache = Arc::new(Cache::<String>::new(5, 1));
        let mut handles = Vec::<thread::JoinHandle<()>>::new();
        for i in 0..10 {
            let cur_cache = Arc::clone(&lfu_cache);
            let data = "abcdefghijklmnopqrstwxyzABCDEFGHIJKLMNOPQRSTUWXYZ0123456789()!@#$%^&&*()"
                .as_bytes()
                .to_vec();
            let handle = thread::spawn(move || {
                cur_cache.insert(format!("files/pdfs/p{i}"), &data);
            });
            handles.push(handle);
        }
        for handle in handles {
            let _ = handle.join();
        }
        let mut handles = Vec::<thread::JoinHandle<bool>>::new();
        for i in 0..10 {
            let cur_cache = Arc::clone(&lfu_cache);
            let data = "abcdefghijklmnopqrstwxyzABCDEFGHIJKLMNOPQRSTUWXYZ0123456789()!@#$%^&&*()"
                .as_bytes()
                .to_vec();
            let handle = thread::spawn(move || {
                let res = cur_cache.get(format!("files/pdfs/p{i}"));
                if res.is_none() {
                    return false;
                }
                res.unwrap().val.eq(&data)
            });
            handles.push(handle);
        }
        for handle in handles {
            let val = handle.join();
            assert!(val.is_ok());
            assert!(val.unwrap());
        }
    }
    #[test]
    fn eviction_test() {
        let lfu_cache = Cache::<String>::new(5, 1);
        let data = "abcdefghijklmnopqrstwxyzABCDEFGHIJKLMNOPQRSTUWXYZ0123456789()!@#$%^&&*()"
            .as_bytes()
            .to_vec(); //size : 72B;
        for i in 0..72816 {
            lfu_cache.insert(format!("files/mp4/v{i}.mp4"), &data);
        }
        let res = lfu_cache.get(format!("files/mp4/v0.mp4"));
        assert!(res.is_none(), "FIRST INSERTED NOT EVICTED");
        {
            assert!(
                lfu_cache
                    .key_cache_map
                    .lock()
                    .unwrap()
                    .get(&format!("files/mp4/v0.mp4"))
                    .is_none(),
                "KEY CACHE MAP LAZY EVICTION FAILED."
            );
        }
        _ = lfu_cache.get(format!("files/mp4/v1.mp4")); //Icrease v1 frequency.

        lfu_cache.insert(String::from("files/mp4/v72816.mp4"), &data); //v6 should be evicted.

        let res_v1 = lfu_cache.get(format!("files/mp4/v1.mp4"));
        assert!(res_v1.is_some(), "WRONG KEY EVICTED.");
        assert!(res_v1.unwrap().val.eq(&data), "V1 VALUE DIFFERENT");

        let res_v6 = lfu_cache.get(String::from("files/mp4/v6.mp4"));
        assert!(res_v6.is_none(), "V6 NOT EVICTED");
    }
}
