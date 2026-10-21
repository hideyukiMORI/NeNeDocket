//! 合成ルートが中核へ渡すポートの束（ARC-006）。

use alloc::boxed::Box;

use crate::clock_port::ClockPort;
use crate::journal_port::JournalPort;
use crate::settings_port::SettingsPort;

/// ジャーナル・壁時計・設定。増えたらここに足し、`DocketScreen::open` の引数は増やさない。
pub struct Ports {
    journal: Box<dyn JournalPort>,
    clock: Box<dyn ClockPort>,
    settings: Box<dyn SettingsPort>,
}

impl Ports {
    /// 実装を束ねる。
    #[must_use]
    pub fn new(
        journal: Box<dyn JournalPort>,
        clock: Box<dyn ClockPort>,
        settings: Box<dyn SettingsPort>,
    ) -> Self {
        Self {
            journal,
            clock,
            settings,
        }
    }

    /// ジャーナル（読むだけ）。
    #[must_use]
    pub fn journal(&self) -> &dyn JournalPort {
        self.journal.as_ref()
    }

    /// ジャーナル（追記・同期・読み直し）。
    pub fn journal_mut(&mut self) -> &mut dyn JournalPort {
        self.journal.as_mut()
    }

    /// 壁時計。
    #[must_use]
    pub fn clock(&self) -> &dyn ClockPort {
        self.clock.as_ref()
    }

    /// 設定（書く）。
    pub fn settings_mut(&mut self) -> &mut dyn SettingsPort {
        self.settings.as_mut()
    }
}

impl core::fmt::Debug for Ports {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("Ports { journal, clock, settings }")
    }
}
