//! 担当の欄の候補（Issue #133）。参加者と、台帳で既に使われている担当を出し、自由に書くこともできる。
//! 「alice」と「alicemori」のような書き分けを、打つ前に見える候補で防ぐ。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::key_intent::KeyIntent;

/// 候補の数の上限（欄の下に並べる）。
pub const LIMIT: usize = 6;

/// 鍵で選んでいる候補と、台帳で使われている担当の覚え（台帳の世代ごと）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssigneePicker {
    highlighted: Option<usize>,
    known: Option<(u64, Vec<String>)>,
}

impl AssigneePicker {
    /// 何も選んでおらず、何も覚えていない。
    #[must_use]
    pub const fn closed() -> Self {
        Self {
            highlighted: None,
            known: None,
        }
    }

    /// 台帳の世代が変わったときだけ、使われている担当を数え直す。
    pub fn refresh<F: FnOnce() -> Vec<String>>(&mut self, generation: u64, used: F) {
        if self
            .known
            .as_ref()
            .is_none_or(|(known, _)| *known != generation)
        {
            self.known = Some((generation, used()));
        }
    }

    /// 候補: 参加者、続いて使われている担当。重ねず、打った字を含むもの（大文字と小文字は区別しない）だけ、
    /// 打った字そのものは除く。
    #[must_use]
    pub fn list(&self, members: &[String], typed: &str) -> Vec<String> {
        let typed = typed.trim();
        let wanted = typed.to_lowercase();
        let used = self
            .known
            .as_ref()
            .map_or(&[][..], |(_, used)| used.as_slice());
        let mut list: Vec<String> = Vec::new();
        for name in members.iter().chain(used) {
            let name = name.trim();
            let fits = !name.is_empty()
                && name != typed
                && name.to_lowercase().contains(&wanted)
                && !list.iter().any(|known| known == name);
            if fits {
                list.push(String::from(name));
            }
            if list.len() == LIMIT {
                break;
            }
        }
        list
    }

    /// 鍵で選んでいる候補の番号。
    #[must_use]
    pub const fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// 上下で候補を選ぶ（端で止まる・上の端から上で選びを外す）。ほかの鍵は選びを外す。`count` は候補の数。
    pub fn key(&mut self, key: KeyIntent, count: usize) {
        self.highlighted = match (key, self.highlighted) {
            (KeyIntent::Down, None) if count > 0 => Some(0),
            (KeyIntent::Down, Some(index)) => Some((index + 1).min(count.saturating_sub(1))),
            (KeyIntent::Up, Some(index)) => index.checked_sub(1),
            (
                KeyIntent::Up
                | KeyIntent::Down
                | KeyIntent::Char(_)
                | KeyIntent::Move { .. }
                | KeyIntent::Backspace
                | KeyIntent::Delete
                | KeyIntent::SelectAll
                | KeyIntent::Enter
                | KeyIntent::Escape
                | KeyIntent::Tab
                | KeyIntent::BackTab
                | KeyIntent::Undo
                | KeyIntent::Redo
                | KeyIntent::NewItem
                | KeyIntent::Find
                | KeyIntent::Help,
                _,
            ) => None,
        };
    }

    /// 選びを外す（候補を選んだ・欄を離れた）。
    pub const fn reset(&mut self) {
        self.highlighted = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| String::from(*name)).collect()
    }

    #[test]
    fn members_come_first_then_used_names_without_repeats_and_filtered_by_the_typed_text() {
        let mut picker = AssigneePicker::closed();
        picker.refresh(1, || names(&["alicemori", "Bob", "alice", ""]));
        let members = names(&["alice", "carol"]);
        assert_eq!(
            picker.list(&members, ""),
            names(&["alice", "carol", "alicemori", "Bob"])
        );
        assert_eq!(picker.list(&members, "ALI"), names(&["alice", "alicemori"]));
        // 打った字そのものは出さない。
        assert_eq!(picker.list(&members, "alice"), names(&["alicemori"]));
        // 世代が同じなら数え直さない。
        picker.refresh(1, || names(&["dave"]));
        assert_eq!(picker.list(&[], "dave"), Vec::<String>::new());
        picker.refresh(2, || names(&["dave"]));
        assert_eq!(picker.list(&[], "d"), names(&["dave"]));
    }

    #[test]
    fn the_list_stops_at_the_limit() {
        let mut picker = AssigneePicker::closed();
        picker.refresh(1, || (0_u32..10).map(|n| alloc::format!("n{n}")).collect());
        assert_eq!(picker.list(&[], "n").len(), LIMIT);
    }

    #[test]
    fn up_and_down_walk_the_candidates_and_other_keys_drop_the_choice() {
        let mut picker = AssigneePicker::closed();
        picker.key(KeyIntent::Down, 0);
        assert_eq!(picker.highlighted(), None);
        picker.key(KeyIntent::Down, 2);
        picker.key(KeyIntent::Down, 2);
        picker.key(KeyIntent::Down, 2);
        assert_eq!(picker.highlighted(), Some(1));
        picker.key(KeyIntent::Up, 2);
        assert_eq!(picker.highlighted(), Some(0));
        picker.key(KeyIntent::Up, 2);
        assert_eq!(picker.highlighted(), None);
        picker.key(KeyIntent::Down, 2);
        picker.key(KeyIntent::Char('x'), 2);
        assert_eq!(picker.highlighted(), None);
        picker.key(KeyIntent::Down, 2);
        picker.reset();
        assert_eq!(picker.highlighted(), None);
    }
}
