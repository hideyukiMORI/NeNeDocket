//! キーボード配列。keycode と Shift の有無から keysym を引く。

use x11rb::protocol::xproto::{GetKeyboardMappingReply, Keycode};

/// `GetKeyboardMapping` の応答を、keycode ごとの keysym の列として持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    first: Keycode,
    per_keycode: usize,
    keysyms: Vec<u32>,
}

impl KeyMap {
    /// 応答から作る。`first` はその応答の最初の keycode。
    #[must_use]
    pub fn from_mapping(first: Keycode, mapping: &GetKeyboardMappingReply) -> Self {
        Self {
            first,
            per_keycode: usize::from(mapping.keysyms_per_keycode.max(1)),
            keysyms: mapping.keysyms.clone(),
        }
    }

    /// keycode の keysym。Shift が押されていれば 2 列目、無ければ 1 列目。
    /// 2 列目が空（0）なら 1 列目に落ちる。配列の外なら `None`。
    #[must_use]
    pub fn keysym(&self, keycode: Keycode, shifted: bool) -> Option<u32> {
        let row = usize::from(keycode.checked_sub(self.first)?) * self.per_keycode;
        let plain = self
            .keysyms
            .get(row)
            .copied()
            .filter(|keysym| *keysym != 0)?;
        if !shifted {
            return Some(plain);
        }
        let shifted_keysym = self.keysyms.get(row + 1).copied().unwrap_or(0);
        Some(if shifted_keysym == 0 {
            plain
        } else {
            shifted_keysym
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> KeyMap {
        let mapping = GetKeyboardMappingReply {
            keysyms_per_keycode: 2,
            sequence: 0,
            keysyms: vec![0, 0, 0xff1b, 0, 0x33, 0x23, 0x61, 0],
        };
        KeyMap::from_mapping(8, &mapping)
    }

    #[test]
    fn looks_up_plain_and_shifted_columns() {
        assert_eq!(map().keysym(9, false), Some(0xff1b));
        assert_eq!(map().keysym(10, false), Some(0x33));
        assert_eq!(map().keysym(10, true), Some(0x23));
        assert_eq!(map().keysym(11, true), Some(0x61));
    }

    #[test]
    fn unknown_keycodes_give_nothing() {
        assert_eq!(map().keysym(8, false), None);
        assert_eq!(map().keysym(7, false), None);
        assert_eq!(map().keysym(200, false), None);
    }
}
