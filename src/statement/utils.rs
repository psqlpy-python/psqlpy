use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn hash_str(string: &String) -> u64 {
    let mut hasher = DefaultHasher::new();
    string.hash(&mut hasher);

    hasher.finish()
}
