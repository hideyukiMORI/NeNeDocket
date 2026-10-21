//! 添付の画像の復号（FR-022・ADR 0018）。PNG と JPEG のバイト列を `Bitmap`（RGBA）にし、大きければ縮める。
//! ファイルも時計も読まない（入力はバイト列だけ）。外部クレートは `png` と `jpeg-decoder` だけ（ARC-013）。

pub mod downscale;
pub mod image_decoding;
pub mod media_failure;
