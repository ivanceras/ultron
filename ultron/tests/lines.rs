use ropey::Rope;
use ropey::RopeSlice;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn test_hash() {
    let mut hasher = DefaultHasher::new();
    let line = "The quick brown fox jumps over the lazy dog";
    line.hash(&mut hasher);
    let hash = hasher.finish();
    println!("Hash is {:x}", hash);
    assert_eq!(0x8dd2037fb281e249, hash);
}

#[test]
fn test_ropey_slice() {
    let content = "The quick brown fox jumps over the lazy dog";
    let rope = Rope::from_str(content);
    let line: RopeSlice = rope.lines_at(0).next().unwrap();
    assert_eq!(content, line);

    let chars: Vec<char> = line.chars().collect();
    let mut hasher = DefaultHasher::new();
    chars.hash(&mut hasher);
    let hash = hasher.finish();

    assert_eq!(6260688385371045911, hash);
}
