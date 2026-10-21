//! XDND の `text/uri-list` を読む。`file://` の URI だけをパスにする（`%20` などは戻す）。`#` の行と空行は飛ばす。

/// `file:///a/b%20c` → `/a/b c`。`file://host/path` は host を落とす。他の URI は飛ばす。
pub(crate) fn file_paths(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim_end_matches('\r').trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.strip_prefix("file://"))
        .filter_map(|rest| {
            if rest.starts_with('/') {
                Some(rest)
            } else {
                rest.find('/').and_then(|slash| rest.get(slash..))
            }
        })
        .map(percent_decoded)
        .collect()
}

/// `%XX` をバイトに戻す。壊れた `%` はそのまま。UTF-8 でない並びは U+FFFD。
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0_usize;
    while let Some(&byte) = bytes.get(index) {
        let decoded = (byte == b'%')
            .then(|| bytes.get(index + 1..index + 3))
            .flatten()
            .and_then(|pair| core::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        if let Some(value) = decoded {
            out.push(value);
            index += 3;
        } else {
            out.push(byte);
            index += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uris_become_paths_and_others_are_skipped() {
        let text = "# comment\r\nfile:///tmp/a%20b.png\r\nhttp://example.com/x\r\nfile://localhost/etc/hosts\r\n\r\nfile:///%E6%97%A5%E6%9C%AC.txt\n";
        assert_eq!(
            file_paths(text),
            ["/tmp/a b.png", "/etc/hosts", "/日本.txt"]
        );
        assert_eq!(file_paths("file://nohost"), Vec::<String>::new());
        assert_eq!(percent_decoded("100%"), "100%");
        assert_eq!(percent_decoded("%zz%4"), "%zz%4");
    }
}
