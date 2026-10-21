//! 文字列をレコードの 1 欄に入れるための逃がし。改行・タブ・区切りの `,` を `\` で逃がす。

/// 逃がす。`\` → `\\`、改行 → `\n`、復帰 → `\r`、タブ → `\t`、`,` → `\,`。
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ',' => out.push_str("\\,"),
            other => out.push(other),
        }
    }
    out
}

/// 戻す。`\` の後に知らない字があれば `None`。
pub(crate) fn unescape(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        out.push(match characters.next()? {
            '\\' => '\\',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            ',' => ',',
            _ => return None,
        });
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_special_character() {
        let text = "a\\b\nc\rd\te,f 日本";
        let escaped = escape(text);
        assert!(!escaped.contains('\n') && !escaped.contains('\t'));
        assert!(escaped.contains("\\,"));
        assert_eq!(unescape(&escaped).as_deref(), Some(text));
        assert_eq!(unescape("plain").as_deref(), Some("plain"));
    }

    #[test]
    fn unknown_escapes_and_a_trailing_backslash_are_rejected() {
        assert_eq!(unescape("a\\x"), None);
        assert_eq!(unescape("a\\"), None);
    }
}
