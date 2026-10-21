//! 画素の列（`SharedBitmap`）を cairo の `ImageSurface` にした覚え。ID が鍵で、毎フレーム作り直さない（FR-022・大容量でも軽い）。

use std::collections::BTreeMap;

use cairo::{Format, ImageSurface};
use nene_docket_core::shared_bitmap::SharedBitmap;

use crate::ui_failure::UiFailure;

/// 覚えておく数の上限。超えたら全部忘れる（添付を次々に見ても記憶が増え続けない）。
const LIMIT: usize = 8;

/// ID → surface。
#[derive(Debug)]
pub struct SurfaceCache {
    surfaces: BTreeMap<u64, ImageSurface>,
}

impl SurfaceCache {
    /// 空。
    #[must_use]
    pub fn empty() -> Self {
        Self {
            surfaces: BTreeMap::new(),
        }
    }

    /// この画素の列の surface。無ければ作る（RGBA → α を掛けた BGRA）。
    ///
    /// # Errors
    ///
    /// cairo が surface を作れない（大きさの上限）。
    pub fn surface_for(&mut self, bitmap: &SharedBitmap) -> Result<&ImageSurface, UiFailure> {
        if !self.surfaces.contains_key(&bitmap.id()) {
            if self.surfaces.len() >= LIMIT {
                self.surfaces.clear();
            }
            let pixels = bitmap.bitmap();
            let width =
                i32::try_from(pixels.width()).map_err(|_error| UiFailure::GeometryOutOfRange)?;
            let height =
                i32::try_from(pixels.height()).map_err(|_error| UiFailure::GeometryOutOfRange)?;
            let stride = width.checked_mul(4).ok_or(UiFailure::GeometryOutOfRange)?;
            let surface = ImageSurface::create_for_data(
                pixels.premultiplied_bgra(),
                Format::ARgb32,
                width,
                height,
                stride,
            )?;
            self.surfaces.insert(bitmap.id(), surface);
        }
        self.surfaces
            .get(&bitmap.id())
            .ok_or(UiFailure::GeometryOutOfRange)
    }

    /// 覚えている数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    /// 何も覚えていないか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::bitmap::Bitmap;

    use super::*;

    fn shared(id: u64) -> SharedBitmap {
        SharedBitmap::new(id, Bitmap::new(1, 1, vec![255, 0, 0, 255]).unwrap())
    }

    #[test]
    fn surfaces_are_made_once_per_id_and_forgotten_past_the_limit() {
        let mut cache = SurfaceCache::empty();
        assert!(cache.is_empty());
        let first = shared(1);
        cache.surface_for(&first).unwrap();
        cache.surface_for(&first).unwrap();
        assert_eq!(cache.len(), 1);
        for id in 2..=u64::try_from(LIMIT).unwrap() {
            cache.surface_for(&shared(id)).unwrap();
        }
        assert_eq!(cache.len(), LIMIT);
        cache.surface_for(&shared(99)).unwrap();
        assert_eq!(cache.len(), 1);
    }
}
