//! 合成ルートが中核へ渡すポートの束（ARC-006）。

use alloc::boxed::Box;

use crate::clock_port::ClockPort;
use crate::journal_port::JournalPort;
use crate::opener_port::OpenerPort;
use crate::settings_port::SettingsPort;

/// ジャーナル・壁時計・設定・OS のアプリで開く。増えたらここに足し、`DocketScreen::open` の引数は増やさない。
pub struct Ports {
    journal: Box<dyn JournalPort>,
    clock: Box<dyn ClockPort>,
    settings: Box<dyn SettingsPort>,
    opener: Box<dyn OpenerPort>,
}

impl Ports {
    /// 実装を束ねる。
    #[must_use]
    pub fn new(
        journal: Box<dyn JournalPort>,
        clock: Box<dyn ClockPort>,
        settings: Box<dyn SettingsPort>,
        opener: Box<dyn OpenerPort>,
    ) -> Self {
        Self {
            journal,
            clock,
            settings,
            opener,
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

    /// OS の既定のアプリで開く（FR-022）。
    #[must_use]
    pub fn opener(&self) -> &dyn OpenerPort {
        self.opener.as_ref()
    }
}

impl core::fmt::Debug for Ports {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("Ports { journal, clock, settings, opener }")
    }
}
