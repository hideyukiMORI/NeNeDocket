//! 共有する画素の列（FR-022）。描画命令は毎フレーム複製されるので、画素は `Arc` で持ち、同じかどうかは ID で決める。
//! ID は中身から決める（添付の SHA-256 の先頭）ので、UI は ID でキャッシュできる。

use alloc::sync::Arc;

use crate::bitmap::Bitmap;

/// ID と画素。
#[derive(Clone)]
pub struct SharedBitmap {
    id: u64,
    bitmap: Arc<Bitmap>,
}

impl SharedBitmap {
    /// 値から。
    #[must_use]
    pub fn new(id: u64, bitmap: Bitmap) -> Self {
        Self {
            id,
            bitmap: Arc::new(bitmap),
        }
    }

    /// ID（UI のキャッシュの鍵）。
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// 画素。
    #[must_use]
    pub fn bitmap(&self) -> &Bitmap {
        &self.bitmap
    }
}

impl PartialEq for SharedBitmap {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SharedBitmap {}

impl core::fmt::Debug for SharedBitmap {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "SharedBitmap #{} ({}×{})",
            self.id,
            self.bitmap.width(),
            self.bitmap.height()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_is_by_id_and_the_pixels_are_shared() {
        let pixels = Bitmap::new(1, 1, alloc::vec![1, 2, 3, 4]).unwrap();
        let first = SharedBitmap::new(7, pixels.clone());
        let twin = first.clone();
        let other = SharedBitmap::new(8, pixels);
        assert_eq!(first, twin);
        assert_ne!(first, other);
        assert_eq!(first.id(), 7);
        assert_eq!(first.bitmap().pixel(0, 0), Some([1, 2, 3, 4]));
        assert_eq!(alloc::format!("{first:?}"), "SharedBitmap #7 (1×1)");
    }
}
