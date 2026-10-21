//! PNG / JPEG のバイト列を RGBA の `Bitmap` に。先頭のバイトで形を見分ける。

use jpeg_decoder::PixelFormat;
use nene_docket_core::bitmap::Bitmap;
use png::{ColorType, Transformations};

use crate::downscale;
use crate::media_failure::MediaFailure;

/// 復号して、長い辺が `max_side` に収まるように縮める。
///
/// # Errors
///
/// 知らない形・壊れている・大きさが扱えない。
pub fn decode(bytes: &[u8], max_side: u32) -> Result<Bitmap, MediaFailure> {
    let bitmap = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png(bytes)?
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        decode_jpeg(bytes)?
    } else {
        return Err(MediaFailure::UnknownFormat);
    };
    let factor = downscale::factor_for(bitmap.width(), bitmap.height(), max_side);
    Ok(downscale::shrink(&bitmap, factor))
}

/// PNG。色の種類とビット深度は 8 ビットに揃えてから RGBA にする。
fn decode_png(bytes: &[u8]) -> Result<Bitmap, MediaFailure> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|_error| MediaFailure::Broken)?;
    let mut buffer = vec![0_u8; reader.output_buffer_size().ok_or(MediaFailure::BadSize)?];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|_error| MediaFailure::Broken)?;
    let pixels = buffer
        .get(..info.buffer_size())
        .ok_or(MediaFailure::Broken)?;
    let channels = match info.color_type {
        ColorType::Grayscale => 1_usize,
        ColorType::GrayscaleAlpha => 2_usize,
        ColorType::Rgb => 3_usize,
        ColorType::Rgba => 4_usize,
        ColorType::Indexed => return Err(MediaFailure::Broken),
    };
    Bitmap::new(info.width, info.height, to_rgba(pixels, channels)).ok_or(MediaFailure::BadSize)
}

/// JPEG。灰色・RGB・CMYK を RGBA にする。
fn decode_jpeg(bytes: &[u8]) -> Result<Bitmap, MediaFailure> {
    let mut decoder = jpeg_decoder::Decoder::new(bytes);
    let pixels = decoder.decode().map_err(|_error| MediaFailure::Broken)?;
    let info = decoder.info().ok_or(MediaFailure::Broken)?;
    let rgba = match info.pixel_format {
        PixelFormat::L8 => to_rgba(&pixels, 1),
        PixelFormat::L16 => to_rgba(&halved(&pixels), 1),
        PixelFormat::RGB24 => to_rgba(&pixels, 3),
        PixelFormat::CMYK32 => cmyk_to_rgba(&pixels),
    };
    Bitmap::new(u32::from(info.width), u32::from(info.height), rgba).ok_or(MediaFailure::BadSize)
}

/// 1（灰）・2（灰 ＋ α）・3（RGB）・4（RGBA）チャネルを RGBA に。
fn to_rgba(pixels: &[u8], channels: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() / channels.max(1) * 4);
    for pixel in pixels.chunks_exact(channels.max(1)) {
        let (Some(&first), second, third, fourth) =
            (pixel.first(), pixel.get(1), pixel.get(2), pixel.get(3))
        else {
            continue;
        };
        match channels {
            1 => out.extend_from_slice(&[first, first, first, 255]),
            2 => out.extend_from_slice(&[first, first, first, second.copied().unwrap_or(255)]),
            3 => out.extend_from_slice(&[
                first,
                second.copied().unwrap_or(0),
                third.copied().unwrap_or(0),
                255,
            ]),
            _ => out.extend_from_slice(&[
                first,
                second.copied().unwrap_or(0),
                third.copied().unwrap_or(0),
                fourth.copied().unwrap_or(255),
            ]),
        }
    }
    out
}

/// 16 ビットの灰（ビッグエンディアン）を上位バイトだけに。
fn halved(pixels: &[u8]) -> Vec<u8> {
    pixels
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[high, _low]| *high)
        .collect()
}

/// CMYK（Adobe の JPEG は反転している）を RGB に。
fn cmyk_to_rgba(pixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len());
    for [cyan, magenta, yellow, key] in pixels.as_chunks::<4>().0 {
        let convert =
            |value: u8| u8::try_from(u32::from(value) * u32::from(*key) / 255).unwrap_or(u8::MAX);
        out.extend_from_slice(&[convert(*cyan), convert(*magenta), convert(*yellow), 255]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `png` の符号化で作る（復号の相手が同じクレートでも、形の往復は確かめられる）。
    fn png_bytes(width: u32, height: u32, color: ColorType, pixels: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(color);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(pixels).unwrap();
        }
        bytes
    }

    #[test]
    fn png_with_alpha_rgb_and_gray_decode_to_rgba() {
        let rgba = decode(
            &png_bytes(2, 1, ColorType::Rgba, &[255, 0, 0, 128, 0, 255, 0, 255]),
            2048,
        )
        .unwrap();
        assert_eq!(rgba.pixel(0, 0), Some([255, 0, 0, 128]));
        assert_eq!(rgba.pixel(1, 0), Some([0, 255, 0, 255]));
        let rgb = decode(&png_bytes(1, 1, ColorType::Rgb, &[1, 2, 3]), 2048).unwrap();
        assert_eq!(rgb.pixel(0, 0), Some([1, 2, 3, 255]));
        let gray = decode(&png_bytes(1, 2, ColorType::Grayscale, &[9, 200]), 2048).unwrap();
        assert_eq!(gray.pixel(0, 1), Some([200, 200, 200, 255]));
        let gray_alpha =
            decode(&png_bytes(1, 1, ColorType::GrayscaleAlpha, &[7, 8]), 2048).unwrap();
        assert_eq!(gray_alpha.pixel(0, 0), Some([7, 7, 7, 8]));
    }

    #[test]
    fn a_large_png_is_shrunk_to_the_limit_and_bad_input_is_refused() {
        let wide = decode(&png_bytes(8, 2, ColorType::Rgb, &[50_u8; 8 * 2 * 3]), 4).unwrap();
        assert_eq!((wide.width(), wide.height()), (4, 1));
        assert_eq!(wide.pixel(3, 0), Some([50, 50, 50, 255]));
        assert_eq!(
            decode(b"not an image", 2048),
            Err(MediaFailure::UnknownFormat)
        );
        assert_eq!(
            decode(b"\x89PNG\r\n\x1a\nbroken", 2048),
            Err(MediaFailure::Broken)
        );
        assert_eq!(
            decode(&[0xFF, 0xD8, 0xFF, 0x00], 2048),
            Err(MediaFailure::Broken)
        );
    }

    #[test]
    fn a_jpeg_fixture_decodes_to_its_colour() {
        // ImageMagick: convert -size 8x8 xc:'#ff0000' -quality 95 red.jpg
        let bitmap = decode(include_bytes!("../fixtures/red-8x8.jpg"), 2048).unwrap();
        assert_eq!((bitmap.width(), bitmap.height()), (8, 8));
        let [red, green, blue, alpha] = bitmap.pixel(4, 4).unwrap();
        assert!(
            red > 240 && green < 16 && blue < 16 && alpha == 255,
            "{red} {green} {blue}"
        );
        let gray = decode(include_bytes!("../fixtures/gray-4x4.jpg"), 2048).unwrap();
        assert_eq!((gray.width(), gray.height()), (4, 4));
        let [r, g, b, _] = gray.pixel(1, 1).unwrap();
        assert!(
            (r == g && g == b) || (r.abs_diff(g) < 3 && g.abs_diff(b) < 3),
            "{r} {g} {b}"
        );
    }
}
