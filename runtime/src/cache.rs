use std::collections::HashMap;
use std::hash::Hash;

pub type ModuleId = String;

pub trait ModuleCache<I: Hash + Eq + PartialEq, M> {
    fn get(&self, key: &I) -> Option<&M>;
    fn insert(&mut self, key: I, data: M) -> Option<M>;
}

pub struct NoCache {}

impl ModuleCache<ModuleId, Vec<u8>> for NoCache {
    fn get(&self, _key: &ModuleId) -> Option<&Vec<u8>> {
        None
    }

    fn insert(&mut self, _id: ModuleId, _data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Default)]
pub struct Memory {
    inner: HashMap<ModuleId, Vec<u8>>
}

impl ModuleCache<ModuleId, Vec<u8>> for Memory {
    fn get(&self, key: &ModuleId) -> Option<&Vec<u8>> {
        self.inner.get(key)
    }

    fn insert(&mut self, id: ModuleId, data: Vec<u8>) -> Option<Vec<u8>> {
        self.inner.insert(id, data)
    }
}

// pub struct ModuleCache {

    // TODO: use tokio to prevent blocking
    // pub fn save_to_disk(&self, name: &str, data: &Vec<u8>) {
    //     let file_name = name.replace("/", "_");
    //     let path = Path::new(&self.cache_dir).join(file_name);
    //     write(path, data).unwrap()
    // }
// }