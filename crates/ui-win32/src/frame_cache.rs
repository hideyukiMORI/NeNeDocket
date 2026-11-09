//! 2 フレーム分だけ覚える表（Issue #112）。描くたびに作り直していた DirectWrite の文字の配置を、同じ文字列なら使い回す。
//! 前のフレームでも今のフレームでも使わなかったものは捨てるので、覚える数は 2 フレームで描いた分を超えない。
//! OS に触れないので Linux でもテストする。

use std::collections::BTreeMap;

/// 今のフレームで使ったものと、前のフレームで使ったもの。
#[derive(Debug)]
pub struct FrameCache<K: Ord, V: Clone> {
    current: BTreeMap<K, V>,
    previous: BTreeMap<K, V>,
}

impl<K: Ord, V: Clone> FrameCache<K, V> {
    /// 何も覚えていない。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            current: BTreeMap::new(),
            previous: BTreeMap::new(),
        }
    }

    /// 新しいフレームを始める。前の前のフレームから使っていないものを捨てる。
    pub fn begin_frame(&mut self) {
        self.previous = core::mem::take(&mut self.current);
    }

    /// 全部忘れる（字の形式が変わった）。
    pub fn clear(&mut self) {
        self.current.clear();
        self.previous.clear();
    }

    /// 覚えていれば写しを返し、今のフレームで使ったことにする。
    pub fn get(&mut self, key: K) -> Option<V> {
        if let Some(value) = self.current.get(&key) {
            return Some(value.clone());
        }
        let value = self.previous.remove(&key)?;
        self.current.insert(key, value.clone());
        Some(value)
    }

    /// 今のフレームで作ったものを覚える。
    pub fn keep(&mut self, key: K, value: V) {
        self.current.insert(key, value);
    }

    /// 覚えている数（今と前の和）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.current.len().saturating_add(self.previous.len())
    }

    /// 何も覚えていない。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_used_in_either_of_the_last_two_frames_is_kept_and_older_ones_go() {
        let mut cache: FrameCache<String, u32> = FrameCache::empty();
        cache.keep(String::from("a"), 1);
        cache.keep(String::from("b"), 2);
        cache.begin_frame();
        assert_eq!(cache.get(String::from("a")), Some(1));
        assert_eq!(cache.len(), 2);
        cache.begin_frame();
        assert_eq!(
            cache.get(String::from("b")),
            None,
            "unused for a whole frame"
        );
        assert_eq!(cache.get(String::from("a")), Some(1));
        assert_eq!(cache.get(String::from("a")), Some(1));
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}
