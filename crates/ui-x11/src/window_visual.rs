//! 窓に使う visual。根の窓と同じ depth 24 の `TrueColor` だけを使う（FR-018・透明は要らない）。

use x11rb::protocol::xproto::{Screen, VisualClass, Visualid};

/// depth と visual と、それに合う cairo の画素形式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowVisual {
    depth: u8,
    visual: Visualid,
    format: cairo::Format,
}

impl WindowVisual {
    /// 根の窓と同じ visual。depth 24 の `TrueColor` のときだけ cairo の `Rgb24` と並びが合う。
    #[must_use]
    pub fn root(screen: &Screen) -> Option<Self> {
        let true_color = screen
            .allowed_depths
            .iter()
            .filter(|depth| depth.depth == screen.root_depth)
            .flat_map(|depth| depth.visuals.iter())
            .any(|visual| {
                visual.visual_id == screen.root_visual && visual.class == VisualClass::TRUE_COLOR
            });
        (screen.root_depth == 24 && true_color).then_some(Self {
            depth: 24,
            visual: screen.root_visual,
            format: cairo::Format::Rgb24,
        })
    }

    /// depth。
    #[must_use]
    pub const fn depth(self) -> u8 {
        self.depth
    }

    /// visual の ID。
    #[must_use]
    pub const fn visual(self) -> Visualid {
        self.visual
    }

    /// cairo の画素形式。
    #[must_use]
    pub const fn format(self) -> cairo::Format {
        self.format
    }
}
