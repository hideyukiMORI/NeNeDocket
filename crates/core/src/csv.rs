//! CSV / TSV の読み（FR-022）。区切り・`"` の囲み（中の `""` は `"`・囲みの中の改行はそのまま）・`\r\n` と `\n`。
//! 先頭行を見出しにするかは呼び手が決める。壊れた形でも落とさず、読めたところまでを返す。

use alloc::string::String;
use alloc::vec::Vec;

/// 文を行と欄に分ける。最後の改行の後の空の行は数えない。
#[must_use]
pub fn parse(text: &str, separator: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if quoted {
            quoted = quoted_character(character, &mut chars, &mut field);
        } else if character == '"' && field.is_empty() {
            quoted = true;
        } else if character == separator {
            row.push(core::mem::take(&mut field));
        } else if character == '\n' {
            row.push(core::mem::take(&mut field));
            rows.push(core::mem::take(&mut row));
        } else if character != '\r' || chars.peek() != Some(&'\n') {
            field.push(character);
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// 囲みの中の 1 字。`""` は `"`、1 つの `"` で囲みが閉じる。まだ囲みの中なら真。
fn quoted_character(
    character: char,
    chars: &mut core::iter::Peekable<core::str::Chars<'_>>,
    field: &mut String,
) -> bool {
    if character != '"' {
        field.push(character);
        return true;
    }
    if chars.peek() == Some(&'"') {
        chars.next();
        field.push('"');
        return true;
    }
    false
}

/// 列の数（いちばん欄の多い行）。
#[must_use]
pub fn column_count(rows: &[Vec<String>]) -> usize {
    rows.iter().map(Vec::len).max().unwrap_or(0)
}

/// 列ごとの、いちばん広い欄の幅（列の幅を決める元）。ASCII は 1、それ以外（全角など）は 2 と数える。
#[must_use]
pub fn widest(rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths = alloc::vec![0_usize; column_count(rows)];
    for row in rows {
        for (width, field) in widths.iter_mut().zip(row) {
            *width = (*width).max(cells_of(field));
        }
    }
    widths
}

/// 欄の幅（ASCII 1・他 2）。
fn cells_of(field: &str) -> usize {
    field
        .chars()
        .map(|character| if character.is_ascii() { 1 } else { 2 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(fields: &[&str]) -> Vec<String> {
        fields.iter().map(|field| String::from(*field)).collect()
    }

    #[test]
    fn separators_quotes_and_line_endings_are_handled() {
        let text = "名前,数\r\n\"a, b\",1\n\"say \"\"hi\"\"\",2\n\"two\nlines\",3\n";
        let rows = parse(text, ',');
        assert_eq!(
            rows,
            [
                strings(&["名前", "数"]),
                strings(&["a, b", "1"]),
                strings(&["say \"hi\"", "2"]),
                strings(&["two\nlines", "3"]),
            ]
        );
        assert_eq!(column_count(&rows), 2);
        // 「名前」は全角 2 字で 4、「数」は 2。いちばん広いのは `two\nlines` の 9。
        assert_eq!(widest(&rows), [9, 2]);
        assert_eq!(widest(&[strings(&["名前", "x"])]), [4, 1]);
    }

    #[test]
    fn a_last_line_without_a_newline_and_tabs_count_too() {
        assert_eq!(
            parse("a\tb\n1\t", '\t'),
            [strings(&["a", "b"]), strings(&["1", ""])]
        );
        assert_eq!(parse("", ','), Vec::<Vec<String>>::new());
        assert_eq!(parse("x", ','), [strings(&["x"])]);
        // 閉じていない囲みは読めたところまで。
        assert_eq!(parse("\"open", ','), [strings(&["open"])]);
        assert_eq!(widest(&[]), Vec::<usize>::new());
    }
}
