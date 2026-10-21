//! `PutImage` の宛先。行ごとの転送で変わらないもの。

use x11rb::connection::{Connection as _, RequestConnection as _};
use x11rb::protocol::xproto::{ConnectionExt as _, Gcontext, ImageFormat, Window};
use x11rb::rust_connection::RustConnection;

use crate::canvas::Canvas;
use crate::ui_failure::UiFailure;

/// 転送先の窓と GC、1 行の幅、depth。
#[derive(Debug, Clone, Copy)]
pub(crate) struct Transfer {
    window: Window,
    gc: Gcontext,
    width: u16,
    depth: u8,
}

impl Transfer {
    /// 宛先を束ねる。
    pub(crate) const fn new(destination: (Window, Gcontext), width: u16, depth: u8) -> Self {
        Self {
            window: destination.0,
            gc: destination.1,
            width,
            depth,
        }
    }

    /// 描き切った画素の列を `PutImage` で転送する（ADR 0002）。大きい画像は行で分ける。
    pub(crate) fn present(
        self,
        connection: &RustConnection,
        canvas: &mut Canvas,
    ) -> Result<(), UiFailure> {
        let stride = canvas.stride()?;
        let rows_per_request = rows_per_request(connection.maximum_request_bytes(), stride);
        canvas.with_pixels(|bytes| -> Result<(), UiFailure> {
            for (index, chunk) in bytes.chunks(rows_per_request * stride).enumerate() {
                let first_row = u16::try_from(index * rows_per_request)
                    .map_err(|_error| UiFailure::GeometryOutOfRange)?;
                let rows = u16::try_from(chunk.len() / stride)
                    .map_err(|_error| UiFailure::GeometryOutOfRange)?;
                self.put_rows(connection, (first_row, rows), chunk)?;
            }
            Ok(())
        })??;
        connection.flush()?;
        Ok(())
    }

    /// `rows.0` 行目から `rows.1` 行分の画素を転送する。
    fn put_rows(
        self,
        connection: &RustConnection,
        rows: (u16, u16),
        bytes: &[u8],
    ) -> Result<(), UiFailure> {
        let (first_row, count) = rows;
        if count == 0 {
            return Ok(());
        }
        let dst_y = i16::try_from(first_row).map_err(|_error| UiFailure::GeometryOutOfRange)?;
        connection.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gc,
            self.width,
            count,
            0,
            dst_y,
            0,
            self.depth,
            bytes,
        )?;
        Ok(())
    }
}

/// 1 回の `PutImage` に入れられる行数。要求の頭（24 バイト）の分を残す。
fn rows_per_request(maximum_request_bytes: usize, stride: usize) -> usize {
    maximum_request_bytes
        .saturating_sub(24)
        .checked_div(stride.max(1))
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_per_request_leaves_room_for_the_header() {
        assert_eq!(rows_per_request(16_384 + 24, 1_696), 9);
        assert_eq!(rows_per_request(10, 1_696), 1);
    }
}
