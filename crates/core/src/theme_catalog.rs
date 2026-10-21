//! 組み込みの配色の表。nene-nib の 125 配色（名前・明暗・本文の 3 色・出典）を写し、台帳の色は `theme_derivation` が導く（ADR 0012）。
//!
//! 🔴 この表は `docs/design/colorschemes.md` と同じ内容で、値は出典（`url`）の版に従う。並びは nene-nib の `BuiltinTheme` のまま。
//! 色の値そのものは著作物ではないが、名前と配色の出典を残す。テーマの実装コード（Vim script・VS Code の JSON）は写さない。

use crate::appearance::Appearance;
use crate::rgb::Rgb;
use crate::theme_seed::ThemeSeed;

/// 組み込みの配色の数。
pub const COUNT: usize = 125;

/// 組み込みの配色。先頭 2 つが `system` の既定（ダーク・ライト）。
pub const SEEDS: [ThemeSeed; COUNT] = [
    ThemeSeed::new(
        "ubuntu-aubergine",
        Appearance::Dark,
        (
            Rgb::new(0x30, 0x0A, 0x24),
            Rgb::new(0xEE, 0xEE, 0xEC),
            Rgb::new(0xE9, 0x54, 0x20),
        ),
        (
            "NeNe Nib",
            "MIT",
            "https://github.com/hideyukiMORI/nene-nib",
        ),
    ),
    ThemeSeed::new(
        "neutral-light",
        Appearance::Light,
        (
            Rgb::new(0xF4, 0xF5, 0xF7),
            Rgb::new(0x1B, 0x1F, 0x24),
            Rgb::new(0xE9, 0x54, 0x20),
        ),
        (
            "NeNe Nib",
            "MIT",
            "https://github.com/hideyukiMORI/nene-nib",
        ),
    ),
    ThemeSeed::new(
        "solarized-dark",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x2B, 0x36),
            Rgb::new(0x83, 0x94, 0x96),
            Rgb::new(0x26, 0x8B, 0xD2),
        ),
        (
            "Ethan Schoonover",
            "MIT",
            "https://ethanschoonover.com/solarized/",
        ),
    ),
    ThemeSeed::new(
        "solarized-light",
        Appearance::Light,
        (
            Rgb::new(0xFD, 0xF6, 0xE3),
            Rgb::new(0x58, 0x6E, 0x75),
            Rgb::new(0x26, 0x8B, 0xD2),
        ),
        (
            "Ethan Schoonover",
            "MIT",
            "https://ethanschoonover.com/solarized/",
        ),
    ),
    ThemeSeed::new(
        "monokai",
        Appearance::Dark,
        (
            Rgb::new(0x27, 0x28, 0x22),
            Rgb::new(0xF8, 0xF8, 0xF2),
            Rgb::new(0xA6, 0xE2, 0x2E),
        ),
        (
            "Wimer Hazenberg",
            "colors used; the name is attributed to the original author",
            "https://github.com/textmate/monokai.tmbundle/blob/master/Themes/Monokai.tmTheme",
        ),
    ),
    ThemeSeed::new(
        "dracula",
        Appearance::Dark,
        (
            Rgb::new(0x28, 0x2A, 0x36),
            Rgb::new(0xF8, 0xF8, 0xF2),
            Rgb::new(0xBD, 0x93, 0xF9),
        ),
        (
            "Zeno Rocha and contributors",
            "MIT",
            "https://spec.draculatheme.com/",
        ),
    ),
    ThemeSeed::new(
        "one-dark",
        Appearance::Dark,
        (
            Rgb::new(0x28, 0x2C, 0x34),
            Rgb::new(0xAB, 0xB2, 0xBF),
            Rgb::new(0x61, 0xAF, 0xEF),
        ),
        ("Atom", "MIT", "https://github.com/atom/one-dark-syntax"),
    ),
    ThemeSeed::new(
        "night-owl",
        Appearance::Dark,
        (
            Rgb::new(0x01, 0x16, 0x27),
            Rgb::new(0xD6, 0xDE, 0xEB),
            Rgb::new(0x82, 0xAA, 0xFF),
        ),
        (
            "Sarah Drasner",
            "MIT",
            "https://github.com/sdras/night-owl-vscode-theme",
        ),
    ),
    ThemeSeed::new(
        "night-owl-light",
        Appearance::Light,
        (
            Rgb::new(0xFB, 0xFB, 0xFB),
            Rgb::new(0x40, 0x3F, 0x53),
            Rgb::new(0x48, 0x76, 0xD6),
        ),
        (
            "Sarah Drasner",
            "MIT",
            "https://github.com/sdras/night-owl-vscode-theme",
        ),
    ),
    ThemeSeed::new(
        "blue",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x87),
            Rgb::new(0xFF, 0xD7, 0x00),
            Rgb::new(0xBC, 0xBC, 0xBC),
        ),
        (
            "Original author Steven Vertigan <steven@vertigan.wattle.id.au>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/blue.vim",
        ),
    ),
    ThemeSeed::new(
        "catppuccin",
        Appearance::Dark,
        (
            Rgb::new(0x1E, 0x1E, 0x2E),
            Rgb::new(0xCD, 0xD6, 0xF4),
            Rgb::new(0xF2, 0xCD, 0xCD),
        ),
        (
            "The Catppuccin Community <catppuccin.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/catppuccin.vim",
        ),
    ),
    ThemeSeed::new(
        "darkblue",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x40),
            Rgb::new(0xC0, 0xC0, 0xC0),
            Rgb::new(0x90, 0xFF, 0xF0),
        ),
        (
            "Original author Bohdan Vlasyuk <bohdan@vstu.edu.ua>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/darkblue.vim",
        ),
    ),
    ThemeSeed::new(
        "default",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x40, 0xFF, 0xFF),
        ),
        (
            "Vim contributors",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/default.vim",
        ),
    ),
    ThemeSeed::new(
        "delek",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x8B, 0x8B),
        ),
        (
            "Original author David Schweikert <david@schweikert.ch>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/delek.vim",
        ),
    ),
    ThemeSeed::new(
        "desert",
        Appearance::Dark,
        (
            Rgb::new(0x33, 0x33, 0x33),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x89, 0xFB, 0x98),
        ),
        (
            "Original author Hans Fugal <hans@fugal.net>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/desert.vim",
        ),
    ),
    ThemeSeed::new(
        "elflord",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0xFF, 0xFF),
            Rgb::new(0x40, 0xFF, 0xFF),
        ),
        (
            "original author Ron Aaron <ron@ronware.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/elflord.vim",
        ),
    ),
    ThemeSeed::new(
        "evening",
        Appearance::Dark,
        (
            Rgb::new(0x33, 0x33, 0x33),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0xFF, 0xFF),
        ),
        (
            "Original author Bram Moolenaar <Bram@vim.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/evening.vim",
        ),
    ),
    ThemeSeed::new(
        "habamax",
        Appearance::Dark,
        (
            Rgb::new(0x1C, 0x1C, 0x1C),
            Rgb::new(0xC7, 0xC7, 0xC7),
            Rgb::new(0x87, 0xAF, 0xAF),
        ),
        (
            "Maxim Kim <habamax@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/habamax.vim",
        ),
    ),
    ThemeSeed::new(
        "industry",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xDA, 0xDA, 0xDA),
            Rgb::new(0xFF, 0x00, 0xFF),
        ),
        (
            "Original author Shian Lee.",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/industry.vim",
        ),
    ),
    ThemeSeed::new(
        "koehler",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x40, 0xFF, 0xFF),
        ),
        (
            "original author Ron Aaron <ron@ronware.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/koehler.vim",
        ),
    ),
    ThemeSeed::new(
        "lunaperche",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xC6, 0xC6, 0xC6),
            Rgb::new(0xE4, 0xE4, 0xE4),
        ),
        (
            "Maxim Kim <habamax@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/lunaperche.vim",
        ),
    ),
    ThemeSeed::new(
        "morning",
        Appearance::Light,
        (
            Rgb::new(0xE4, 0xE4, 0xE4),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x87, 0x87),
        ),
        (
            "Original author Bram Moolenaar <Bram@vim.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/morning.vim",
        ),
    ),
    ThemeSeed::new(
        "murphy",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x87, 0xFF, 0x87),
            Rgb::new(0x00, 0xFF, 0xFF),
        ),
        (
            "Original author Ron Aaron <ron@ronware.org>.",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/murphy.vim",
        ),
    ),
    ThemeSeed::new(
        "novum",
        Appearance::Dark,
        (
            Rgb::new(0x14, 0x16, 0x1B),
            Rgb::new(0xE0, 0xE2, 0xEA),
            Rgb::new(0xA6, 0xDB, 0xFF),
        ),
        (
            "Charan Mandati <charancuz008@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/novum.vim",
        ),
    ),
    ThemeSeed::new(
        "pablo",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0xC0, 0xC0),
        ),
        (
            "Ron Aaron <ron@ronware.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/pablo.vim",
        ),
    ),
    ThemeSeed::new(
        "peachpuff",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xDA, 0xB9),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x8B, 0x8B),
        ),
        (
            "Original author David Ne\\\\v{c}as (Yeti) <yeti@physics.muni.cz>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/peachpuff.vim",
        ),
    ),
    ThemeSeed::new(
        "quiet",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xDA, 0xDA, 0xDA),
            Rgb::new(0xDA, 0xDA, 0xDA),
        ),
        (
            "Maxence Weynans <neutaaaaan@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/quiet.vim",
        ),
    ),
    ThemeSeed::new(
        "retrobox",
        Appearance::Dark,
        (
            Rgb::new(0x1C, 0x1C, 0x1C),
            Rgb::new(0xEB, 0xDB, 0xB2),
            Rgb::new(0x83, 0xA5, 0x98),
        ),
        (
            "Maxim Kim <habamax@gmail.com>, ported from gruvbox8 of Lifepillar <lifepillar@lifepillar.me>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/retrobox.vim",
        ),
    ),
    ThemeSeed::new(
        "ron",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0xFF, 0xFF),
            Rgb::new(0x00, 0xFF, 0xFF),
        ),
        (
            "original author Ron Aaron <ron@ronware.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/ron.vim",
        ),
    ),
    ThemeSeed::new(
        "shine",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x8B, 0x8B),
        ),
        (
            "Original author is Yasuhiro Matsumoto <mattn.jp@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/shine.vim",
        ),
    ),
    ThemeSeed::new(
        "slate",
        Appearance::Dark,
        (
            Rgb::new(0x26, 0x26, 0x26),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0xFF, 0x87, 0x87),
        ),
        (
            "Original maintainer Ralph Amissah <ralph@amissah.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/slate.vim",
        ),
    ),
    ThemeSeed::new(
        "sorbet",
        Appearance::Dark,
        (
            Rgb::new(0x16, 0x18, 0x21),
            Rgb::new(0xDA, 0xDA, 0xDA),
            Rgb::new(0x87, 0xD7, 0x5F),
        ),
        (
            "Maxence Weynans <neutaaaaan@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/sorbet.vim",
        ),
    ),
    ThemeSeed::new(
        "torte",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xCC, 0xCC, 0xCC),
            Rgb::new(0x40, 0xFF, 0xFF),
        ),
        (
            "Original maintainer Thorsten Maerz <info@netztorte.de>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/torte.vim",
        ),
    ),
    ThemeSeed::new(
        "unokai",
        Appearance::Dark,
        (
            Rgb::new(0x28, 0x29, 0x23),
            Rgb::new(0xF8, 0xF8, 0xF2),
            Rgb::new(0x66, 0xD9, 0xEF),
        ),
        (
            "k-37 <60838818+k-37@users.noreply.github.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/unokai.vim",
        ),
    ),
    ThemeSeed::new(
        "wildcharm",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xD0, 0xD0, 0xD0),
            Rgb::new(0xFF, 0x87, 0xFF),
        ),
        (
            "Maxim Kim <habamax@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/wildcharm.vim",
        ),
    ),
    ThemeSeed::new(
        "zaibatsu",
        Appearance::Dark,
        (
            Rgb::new(0x0E, 0x00, 0x24),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x87, 0xFF, 0xFF),
        ),
        (
            "Romain Lafourcade <romainlafourcade@gmail.com>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/zaibatsu.vim",
        ),
    ),
    ThemeSeed::new(
        "zellner",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x00, 0xFF),
        ),
        (
            "Original author Ron Aaron <ron@ronware.org>",
            "Vim license; palette adaptation",
            "https://github.com/vim/vim/blob/99b9dd00cd180bade5b8b08c00e4e8ae2e553dbe/runtime/colors/zellner.vim",
        ),
    ),
    ThemeSeed::new(
        "abyss",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x0C, 0x18),
            Rgb::new(0x66, 0x88, 0xCC),
            Rgb::new(0x59, 0x6F, 0x99),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-abyss/themes/abyss-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-dark-2026",
        Appearance::Dark,
        (
            Rgb::new(0x12, 0x13, 0x14),
            Rgb::new(0xBB, 0xBE, 0xBF),
            Rgb::new(0x2D, 0x6E, 0x8A),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/2026-dark.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-light-2026",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x20, 0x20, 0x20),
            Rgb::new(0x00, 0x69, 0xCC),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/2026-light.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-dark-modern",
        Appearance::Dark,
        (
            Rgb::new(0x1F, 0x1F, 0x1F),
            Rgb::new(0xCC, 0xCC, 0xCC),
            Rgb::new(0x00, 0x78, 0xD4),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/dark_modern.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-dark-plus",
        Appearance::Dark,
        (
            Rgb::new(0x1E, 0x1E, 0x1E),
            Rgb::new(0xD4, 0xD4, 0xD4),
            Rgb::new(0xDC, 0xDC, 0xAA),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/dark_plus.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-dark",
        Appearance::Dark,
        (
            Rgb::new(0x1E, 0x1E, 0x1E),
            Rgb::new(0xD4, 0xD4, 0xD4),
            Rgb::new(0x56, 0x9C, 0xD6),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/dark_vs.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-high-contrast",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0xDC, 0xDC, 0xAA),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/hc_black.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-high-contrast-light",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x29, 0x29, 0x29),
            Rgb::new(0x5E, 0x2C, 0xBC),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/hc_light.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-light-modern",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x3B, 0x3B, 0x3B),
            Rgb::new(0x00, 0x5F, 0xB8),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/light_modern.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-light-plus",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x79, 0x5E, 0x26),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/light_plus.json",
        ),
    ),
    ThemeSeed::new(
        "vscode-light",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x00, 0xFF),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-defaults/themes/light_vs.json",
        ),
    ),
    ThemeSeed::new(
        "kimbie-dark",
        Appearance::Dark,
        (
            Rgb::new(0x22, 0x1A, 0x0F),
            Rgb::new(0xD3, 0xAF, 0x86),
            Rgb::new(0xA5, 0x7A, 0x4C),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-kimbie-dark/themes/kimbie-dark-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "monokai-dimmed",
        Appearance::Dark,
        (
            Rgb::new(0x1E, 0x1E, 0x1E),
            Rgb::new(0xC5, 0xC8, 0xC6),
            Rgb::new(0x36, 0x55, 0xB5),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-monokai-dimmed/themes/dimmed-monokai-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "quiet-light",
        Appearance::Light,
        (
            Rgb::new(0xF5, 0xF5, 0xF5),
            Rgb::new(0x33, 0x33, 0x33),
            Rgb::new(0x97, 0x69, 0xDC),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-quietlight/themes/quietlight-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "red",
        Appearance::Dark,
        (
            Rgb::new(0x39, 0x00, 0x00),
            Rgb::new(0xF8, 0xF8, 0xF8),
            Rgb::new(0xBD, 0x44, 0x44),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-red/themes/Red-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "tomorrow-night-blue",
        Appearance::Dark,
        (
            Rgb::new(0x00, 0x24, 0x51),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0xBB, 0xDA, 0xFF),
        ),
        (
            "Microsoft and theme contributors",
            "MIT",
            "https://github.com/microsoft/vscode/blob/6133b425600558c15ca5238ce483a2f5bdb8db9c/extensions/theme-tomorrow-night-blue/themes/tomorrow-night-blue-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "monalisa",
        Appearance::Dark,
        (
            Rgb::new(0x12, 0x0B, 0x0D),
            Rgb::new(0xF6, 0xD6, 0x66),
            Rgb::new(0xF6, 0xD6, 0x66),
        ),
        (
            "Patrick Dewey and contributors",
            "MIT",
            "https://github.com/ptdewey/monalisa-nvim/blob/2ffe6db37fcad17da0d210f5b3a357712d0b8a2b/fnl/monalisa/init.fnl",
        ),
    ),
    ThemeSeed::new(
        "catppuccin-latte",
        Appearance::Light,
        (
            Rgb::new(0xEF, 0xF1, 0xF5),
            Rgb::new(0x4C, 0x4F, 0x69),
            Rgb::new(0x1E, 0x66, 0xF5),
        ),
        (
            "Catppuccin contributors",
            "MIT",
            "https://github.com/catppuccin/palette/blob/07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd/palette.json",
        ),
    ),
    ThemeSeed::new(
        "catppuccin-frappe",
        Appearance::Dark,
        (
            Rgb::new(0x30, 0x34, 0x46),
            Rgb::new(0xC6, 0xD0, 0xF5),
            Rgb::new(0x8C, 0xAA, 0xEE),
        ),
        (
            "Catppuccin contributors",
            "MIT",
            "https://github.com/catppuccin/palette/blob/07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd/palette.json",
        ),
    ),
    ThemeSeed::new(
        "catppuccin-macchiato",
        Appearance::Dark,
        (
            Rgb::new(0x24, 0x27, 0x3A),
            Rgb::new(0xCA, 0xD3, 0xF5),
            Rgb::new(0x8A, 0xAD, 0xF4),
        ),
        (
            "Catppuccin contributors",
            "MIT",
            "https://github.com/catppuccin/palette/blob/07d02aa110ef9eb7e7427afca5c73ba9cf7f8ebd/palette.json",
        ),
    ),
    ThemeSeed::new(
        "nord",
        Appearance::Dark,
        (
            Rgb::new(0x2E, 0x34, 0x40),
            Rgb::new(0xD8, 0xDE, 0xE9),
            Rgb::new(0x88, 0xC0, 0xD0),
        ),
        (
            "Arctic Ice Studio and Sven Greb",
            "MIT",
            "https://github.com/nordtheme/nord/blob/1cef71605416a222e57225b544540ce0fcec18d4/src/nord.scss",
        ),
    ),
    ThemeSeed::new(
        "gruvbox",
        Appearance::Dark,
        (
            Rgb::new(0x28, 0x28, 0x28),
            Rgb::new(0xEB, 0xDB, 0xB2),
            Rgb::new(0xB8, 0xBB, 0x26),
        ),
        (
            "Pavel Pertsev",
            "MIT",
            "https://github.com/morhetz/gruvbox/blob/ef8864bb42bf244f0295d1c5a403b27e3d139695/colors/gruvbox.vim",
        ),
    ),
    ThemeSeed::new(
        "gruvbox-light",
        Appearance::Light,
        (
            Rgb::new(0xFB, 0xF1, 0xC7),
            Rgb::new(0x3C, 0x38, 0x36),
            Rgb::new(0x79, 0x74, 0x0E),
        ),
        (
            "Pavel Pertsev",
            "MIT",
            "https://github.com/morhetz/gruvbox/blob/ef8864bb42bf244f0295d1c5a403b27e3d139695/colors/gruvbox.vim",
        ),
    ),
    ThemeSeed::new(
        "tokyo-night",
        Appearance::Dark,
        (
            Rgb::new(0x1A, 0x1B, 0x26),
            Rgb::new(0xC0, 0xCA, 0xF5),
            Rgb::new(0x7A, 0xA2, 0xF7),
        ),
        (
            "Folke Lemaitre and Tokyo Night contributors",
            "MIT",
            "https://github.com/folke/tokyonight.nvim/blob/cdc07ac78467a233fd62c493de29a17e0cf2b2b6/lua/tokyonight/colors/night.lua",
        ),
    ),
    ThemeSeed::new(
        "tokyo-night-storm",
        Appearance::Dark,
        (
            Rgb::new(0x24, 0x28, 0x3B),
            Rgb::new(0xC0, 0xCA, 0xF5),
            Rgb::new(0x7A, 0xA2, 0xF7),
        ),
        (
            "Folke Lemaitre and Tokyo Night contributors",
            "MIT",
            "https://github.com/folke/tokyonight.nvim/blob/cdc07ac78467a233fd62c493de29a17e0cf2b2b6/lua/tokyonight/colors/storm.lua",
        ),
    ),
    ThemeSeed::new(
        "tokyo-night-moon",
        Appearance::Dark,
        (
            Rgb::new(0x22, 0x24, 0x36),
            Rgb::new(0xC8, 0xD3, 0xF5),
            Rgb::new(0x82, 0xAA, 0xFF),
        ),
        (
            "Folke Lemaitre and Tokyo Night contributors",
            "MIT",
            "https://github.com/folke/tokyonight.nvim/blob/cdc07ac78467a233fd62c493de29a17e0cf2b2b6/lua/tokyonight/colors/moon.lua",
        ),
    ),
    ThemeSeed::new(
        "tokyo-night-light",
        Appearance::Light,
        (
            Rgb::new(0xE6, 0xE7, 0xED),
            Rgb::new(0x34, 0x3B, 0x59),
            Rgb::new(0xCE, 0xD0, 0xD7),
        ),
        (
            "Enkia",
            "MIT",
            "https://github.com/enkia/tokyo-night-vscode-theme/blob/7c0f11eaef322f293621ca7befe462214b7ea468/themes/tokyo-night-light-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "github-dark",
        Appearance::Dark,
        (
            Rgb::new(0x0D, 0x11, 0x17),
            Rgb::new(0xC9, 0xD1, 0xD9),
            Rgb::new(0xD2, 0xA8, 0xFF),
        ),
        (
            "GitHub and Primer contributors",
            "MIT",
            "https://github.com/primer/github-vscode-theme/blob/cd78e5e4e7bcf132a6f428ae0f32264bb1b729cf/src/theme.js",
        ),
    ),
    ThemeSeed::new(
        "github-light",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x24, 0x29, 0x2F),
            Rgb::new(0x82, 0x50, 0xDF),
        ),
        (
            "GitHub and Primer contributors",
            "MIT",
            "https://github.com/primer/github-vscode-theme/blob/cd78e5e4e7bcf132a6f428ae0f32264bb1b729cf/src/theme.js",
        ),
    ),
    ThemeSeed::new(
        "github-dark-dimmed",
        Appearance::Dark,
        (
            Rgb::new(0x22, 0x27, 0x2E),
            Rgb::new(0xAD, 0xBA, 0xC7),
            Rgb::new(0xDC, 0xBD, 0xFB),
        ),
        (
            "GitHub and Primer contributors",
            "MIT",
            "https://github.com/primer/github-vscode-theme/blob/cd78e5e4e7bcf132a6f428ae0f32264bb1b729cf/src/theme.js",
        ),
    ),
    ThemeSeed::new(
        "one-light",
        Appearance::Light,
        (
            Rgb::new(0xFA, 0xFA, 0xFA),
            Rgb::new(0x38, 0x3A, 0x42),
            Rgb::new(0x40, 0x78, 0xF2),
        ),
        (
            "Atom",
            "MIT",
            "https://github.com/atom/one-light-syntax/blob/d84579027410c576086dfca14d934c4bd74b0438/styles/colors.less",
        ),
    ),
    ThemeSeed::new(
        "intellij-light",
        Appearance::Light,
        (
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0x00, 0x00, 0x00),
        ),
        (
            "JetBrains",
            "Apache-2.0; palette adaptation",
            "https://github.com/JetBrains/intellij-community/blob/8886d47ff4b7a538583b793ba29e9cc642ae01d9/platform/platform-resources/src/DefaultColorSchemesManager.xml",
        ),
    ),
    ThemeSeed::new(
        "darcula",
        Appearance::Dark,
        (
            Rgb::new(0x2B, 0x2B, 0x2B),
            Rgb::new(0xA9, 0xB7, 0xC6),
            Rgb::new(0xFF, 0xC6, 0x6D),
        ),
        (
            "JetBrains",
            "Apache-2.0; palette adaptation",
            "https://github.com/JetBrains/intellij-community/blob/8886d47ff4b7a538583b793ba29e9cc642ae01d9/platform/platform-resources/src/DefaultColorSchemesManager.xml",
        ),
    ),
    ThemeSeed::new(
        "rose-pine",
        Appearance::Dark,
        (
            Rgb::new(0x19, 0x17, 0x24),
            Rgb::new(0xE0, 0xDE, 0xF4),
            Rgb::new(0xEB, 0xBC, 0xBA),
        ),
        (
            "Rosé Pine contributors",
            "MIT",
            "https://github.com/rose-pine/palette/blob/92af52b465ab6e47437aca223c9b8d3009a2023b/palette.json",
        ),
    ),
    ThemeSeed::new(
        "rose-pine-moon",
        Appearance::Dark,
        (
            Rgb::new(0x23, 0x21, 0x36),
            Rgb::new(0xE0, 0xDE, 0xF4),
            Rgb::new(0xEA, 0x9A, 0x97),
        ),
        (
            "Rosé Pine contributors",
            "MIT",
            "https://github.com/rose-pine/palette/blob/92af52b465ab6e47437aca223c9b8d3009a2023b/palette.json",
        ),
    ),
    ThemeSeed::new(
        "rose-pine-dawn",
        Appearance::Light,
        (
            Rgb::new(0xFA, 0xF4, 0xED),
            Rgb::new(0x46, 0x42, 0x61),
            Rgb::new(0xD7, 0x82, 0x7E),
        ),
        (
            "Rosé Pine contributors",
            "MIT",
            "https://github.com/rose-pine/palette/blob/92af52b465ab6e47437aca223c9b8d3009a2023b/palette.json",
        ),
    ),
    ThemeSeed::new(
        "everforest-dark",
        Appearance::Dark,
        (
            Rgb::new(0x2D, 0x35, 0x3B),
            Rgb::new(0xD3, 0xC6, 0xAA),
            Rgb::new(0xA7, 0xC0, 0x80),
        ),
        (
            "Sainnhe Park",
            "MIT",
            "https://github.com/sainnhe/everforest/blob/85a86eb62409e3ec88713bff3d1b9d7374e112e4/autoload/everforest.vim",
        ),
    ),
    ThemeSeed::new(
        "everforest-light",
        Appearance::Light,
        (
            Rgb::new(0xFD, 0xF6, 0xE3),
            Rgb::new(0x5C, 0x6A, 0x72),
            Rgb::new(0x8D, 0xA1, 0x01),
        ),
        (
            "Sainnhe Park",
            "MIT",
            "https://github.com/sainnhe/everforest/blob/85a86eb62409e3ec88713bff3d1b9d7374e112e4/autoload/everforest.vim",
        ),
    ),
    ThemeSeed::new(
        "kanagawa",
        Appearance::Dark,
        (
            Rgb::new(0x1F, 0x1F, 0x28),
            Rgb::new(0xDC, 0xD7, 0xBA),
            Rgb::new(0x7E, 0x9C, 0xD8),
        ),
        (
            "Rebelot and Kanagawa contributors",
            "MIT",
            "https://github.com/rebelot/kanagawa.nvim/blob/bb85e4bfc8d89b0e62c8fa53ccdd13d12e2f77b3/lua/kanagawa/colors.lua",
        ),
    ),
    ThemeSeed::new(
        "kanagawa-dragon",
        Appearance::Dark,
        (
            Rgb::new(0x18, 0x16, 0x16),
            Rgb::new(0xC5, 0xC9, 0xC5),
            Rgb::new(0x8B, 0xA4, 0xB0),
        ),
        (
            "Rebelot and Kanagawa contributors",
            "MIT",
            "https://github.com/rebelot/kanagawa.nvim/blob/bb85e4bfc8d89b0e62c8fa53ccdd13d12e2f77b3/lua/kanagawa/colors.lua",
        ),
    ),
    ThemeSeed::new(
        "kanagawa-lotus",
        Appearance::Light,
        (
            Rgb::new(0xF2, 0xEC, 0xBC),
            Rgb::new(0x54, 0x54, 0x64),
            Rgb::new(0x4D, 0x69, 0x9B),
        ),
        (
            "Rebelot and Kanagawa contributors",
            "MIT",
            "https://github.com/rebelot/kanagawa.nvim/blob/bb85e4bfc8d89b0e62c8fa53ccdd13d12e2f77b3/lua/kanagawa/colors.lua",
        ),
    ),
    ThemeSeed::new(
        "material-oceanic",
        Appearance::Dark,
        (
            Rgb::new(0x26, 0x32, 0x38),
            Rgb::new(0xB0, 0xBE, 0xC5),
            Rgb::new(0x82, 0xAA, 0xFF),
        ),
        (
            "Material Theme / Mallowigi and Equinusocio",
            "published palette colors; names attributed to their authors",
            "https://material-theme.com/docs/reference/color-palette/",
        ),
    ),
    ThemeSeed::new(
        "material-darker",
        Appearance::Dark,
        (
            Rgb::new(0x21, 0x21, 0x21),
            Rgb::new(0xB0, 0xBE, 0xC5),
            Rgb::new(0x82, 0xAA, 0xFF),
        ),
        (
            "Material Theme / Mallowigi and Equinusocio",
            "published palette colors; names attributed to their authors",
            "https://material-theme.com/docs/reference/color-palette/",
        ),
    ),
    ThemeSeed::new(
        "material-lighter",
        Appearance::Light,
        (
            Rgb::new(0xFA, 0xFA, 0xFA),
            Rgb::new(0x54, 0x6E, 0x7A),
            Rgb::new(0x61, 0x82, 0xB8),
        ),
        (
            "Material Theme / Mallowigi and Equinusocio",
            "published palette colors; names attributed to their authors",
            "https://material-theme.com/docs/reference/color-palette/",
        ),
    ),
    ThemeSeed::new(
        "material-palenight",
        Appearance::Dark,
        (
            Rgb::new(0x29, 0x2D, 0x3E),
            Rgb::new(0xA6, 0xAC, 0xCD),
            Rgb::new(0x82, 0xAA, 0xFF),
        ),
        (
            "Material Theme / Mallowigi and Equinusocio",
            "published palette colors; names attributed to their authors",
            "https://material-theme.com/docs/reference/color-palette/",
        ),
    ),
    ThemeSeed::new(
        "material-deep-ocean",
        Appearance::Dark,
        (
            Rgb::new(0x0F, 0x11, 0x1A),
            Rgb::new(0x8F, 0x93, 0xA2),
            Rgb::new(0x82, 0xAA, 0xFF),
        ),
        (
            "Material Theme / Mallowigi and Equinusocio",
            "published palette colors; names attributed to their authors",
            "https://material-theme.com/docs/reference/color-palette/",
        ),
    ),
    ThemeSeed::new(
        "ayu-dark",
        Appearance::Dark,
        (
            Rgb::new(0x10, 0x14, 0x1C),
            Rgb::new(0xBF, 0xBD, 0xB6),
            Rgb::new(0xE6, 0xB4, 0x50),
        ),
        (
            "Ayu Theme and Yihui Hu",
            "MIT",
            "https://github.com/ayu-theme/vscode-ayu/blob/2c6feb7215bf7ca6c8c9a51066f255025df3a5b9/ayu-dark.json",
        ),
    ),
    ThemeSeed::new(
        "ayu-mirage",
        Appearance::Dark,
        (
            Rgb::new(0x24, 0x29, 0x36),
            Rgb::new(0xCC, 0xCA, 0xC2),
            Rgb::new(0xFF, 0xCC, 0x66),
        ),
        (
            "Ayu Theme and Yihui Hu",
            "MIT",
            "https://github.com/ayu-theme/vscode-ayu/blob/2c6feb7215bf7ca6c8c9a51066f255025df3a5b9/ayu-mirage.json",
        ),
    ),
    ThemeSeed::new(
        "ayu-light",
        Appearance::Light,
        (
            Rgb::new(0xFC, 0xFC, 0xFC),
            Rgb::new(0x5C, 0x61, 0x66),
            Rgb::new(0xF2, 0x97, 0x18),
        ),
        (
            "Ayu Theme and Yihui Hu",
            "MIT",
            "https://github.com/ayu-theme/vscode-ayu/blob/2c6feb7215bf7ca6c8c9a51066f255025df3a5b9/ayu-light.json",
        ),
    ),
    ThemeSeed::new(
        "cobalt2",
        Appearance::Dark,
        (
            Rgb::new(0x19, 0x35, 0x49),
            Rgb::new(0xFF, 0xFF, 0xFF),
            Rgb::new(0x0D, 0x3A, 0x58),
        ),
        (
            "Wes Bos",
            "MIT",
            "https://github.com/wesbos/cobalt2-vscode/blob/c4e9574372b85afad1682ed0fdd1ac0411c62512/theme/cobalt2.json",
        ),
    ),
    ThemeSeed::new(
        "synthwave-84",
        Appearance::Dark,
        (
            Rgb::new(0x26, 0x23, 0x35),
            Rgb::new(0xD4, 0xD4, 0xD4),
            Rgb::new(0x1F, 0x21, 0x2B),
        ),
        (
            "Robb Owen",
            "MIT",
            "https://github.com/robb0wen/synthwave-vscode/blob/ecfa2fe1279f7233663fa3f98a96e6756000567b/themes/synthwave-color-theme.json",
        ),
    ),
    ThemeSeed::new(
        "alloy-core",
        Appearance::Dark,
        (
            Rgb::new(0x19, 0x15, 0x1B),
            Rgb::new(0xF3, 0xE9, 0xDC),
            Rgb::new(0xE4, 0xAF, 0x55),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "verdant-rage",
        Appearance::Dark,
        (
            Rgb::new(0x14, 0x20, 0x1B),
            Rgb::new(0xE4, 0xEC, 0xD9),
            Rgb::new(0x9A, 0xCE, 0x6A),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "scarlet-web",
        Appearance::Dark,
        (
            Rgb::new(0x12, 0x1C, 0x31),
            Rgb::new(0xE8, 0xEC, 0xF3),
            Rgb::new(0xEA, 0x71, 0x6D),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "relic-sand",
        Appearance::Light,
        (
            Rgb::new(0xF1, 0xE4, 0xC6),
            Rgb::new(0x3A, 0x2C, 0x20),
            Rgb::new(0x94, 0x50, 0x25),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "soap-rebellion",
        Appearance::Dark,
        (
            Rgb::new(0x1B, 0x25, 0x24),
            Rgb::new(0xE1, 0xE8, 0xD6),
            Rgb::new(0xF3, 0x9B, 0xAD),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "code-rain",
        Appearance::Dark,
        (
            Rgb::new(0x08, 0x13, 0x0E),
            Rgb::new(0xC9, 0xE9, 0xCD),
            Rgb::new(0x75, 0xE5, 0x9A),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "arcane-library",
        Appearance::Dark,
        (
            Rgb::new(0x18, 0x1B, 0x30),
            Rgb::new(0xEA, 0xE2, 0xD1),
            Rgb::new(0xD8, 0xB7, 0x78),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-colorschemes.md",
        ),
    ),
    ThemeSeed::new(
        "royal-kinetic",
        Appearance::Dark,
        (
            Rgb::new(0x11, 0x0E, 0x1C),
            Rgb::new(0xE6, 0xDF, 0xF3),
            Rgb::new(0xB5, 0x82, 0xF5),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "arena-sunset",
        Appearance::Dark,
        (
            Rgb::new(0x19, 0x2C, 0x29),
            Rgb::new(0xEC, 0xE8, 0xCF),
            Rgb::new(0xF0, 0x90, 0x68),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "cosmic-mixtape",
        Appearance::Dark,
        (
            Rgb::new(0x25, 0x16, 0x30),
            Rgb::new(0xEE, 0xE0, 0xF5),
            Rgb::new(0xE6, 0x8B, 0xCD),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "prism-sanctum",
        Appearance::Dark,
        (
            Rgb::new(0x10, 0x28, 0x25),
            Rgb::new(0xE9, 0xE9, 0xD7),
            Rgb::new(0xED, 0xA6, 0x57),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "winter-sentinel",
        Appearance::Dark,
        (
            Rgb::new(0x14, 0x24, 0x31),
            Rgb::new(0xDD, 0xE8, 0xEF),
            Rgb::new(0x8F, 0xB9, 0xCB),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "infinite-dusk",
        Appearance::Dark,
        (
            Rgb::new(0x2C, 0x1D, 0x32),
            Rgb::new(0xF0, 0xE2, 0xE8),
            Rgb::new(0xDD, 0xA2, 0x86),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "stellar-pilot",
        Appearance::Dark,
        (
            Rgb::new(0x14, 0x1E, 0x35),
            Rgb::new(0xED, 0xE7, 0xD9),
            Rgb::new(0xD6, 0xB7, 0x66),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "quantum-honey",
        Appearance::Dark,
        (
            Rgb::new(0x28, 0x22, 0x18),
            Rgb::new(0xEE, 0xE6, 0xD3),
            Rgb::new(0xE4, 0xB6, 0x51),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "crimson-mercenary",
        Appearance::Dark,
        (
            Rgb::new(0x27, 0x16, 0x1B),
            Rgb::new(0xED, 0xE1, 0xDC),
            Rgb::new(0xE5, 0x7D, 0x85),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "last-outlaw",
        Appearance::Dark,
        (
            Rgb::new(0x2A, 0x22, 0x1C),
            Rgb::new(0xE8, 0xDD, 0xC9),
            Rgb::new(0xCA, 0xA0, 0x6A),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "ivory-agent",
        Appearance::Light,
        (
            Rgb::new(0xF2, 0xEF, 0xE8),
            Rgb::new(0x31, 0x36, 0x3B),
            Rgb::new(0xA7, 0x38, 0x37),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "celestial-brass",
        Appearance::Light,
        (
            Rgb::new(0xEE, 0xE8, 0xD8),
            Rgb::new(0x34, 0x36, 0x31),
            Rgb::new(0x9B, 0x78, 0x36),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "amber-haze",
        Appearance::Dark,
        (
            Rgb::new(0x30, 0x1C, 0x18),
            Rgb::new(0xF1, 0xE0, 0xCF),
            Rgb::new(0xEA, 0xA2, 0x5C),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "desert-oracle",
        Appearance::Light,
        (
            Rgb::new(0xEB, 0xE0, 0xC9),
            Rgb::new(0x46, 0x3C, 0x31),
            Rgb::new(0x9D, 0x70, 0x2F),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "chrome-wasteland",
        Appearance::Dark,
        (
            Rgb::new(0x25, 0x32, 0x39),
            Rgb::new(0xE8, 0xE0, 0xC9),
            Rgb::new(0xE6, 0xAC, 0x72),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "neon-grid",
        Appearance::Dark,
        (
            Rgb::new(0x06, 0x1D, 0x27),
            Rgb::new(0xD9, 0xF0, 0xF2),
            Rgb::new(0x6E, 0xDD, 0xEC),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "biolume-forest",
        Appearance::Dark,
        (
            Rgb::new(0x0C, 0x20, 0x38),
            Rgb::new(0xD5, 0xE8, 0xF1),
            Rgb::new(0x69, 0xCD, 0xD9),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "pastel-concierge",
        Appearance::Light,
        (
            Rgb::new(0xF6, 0xE7, 0xE6),
            Rgb::new(0x54, 0x3D, 0x4C),
            Rgb::new(0xA3, 0x4D, 0x70),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "twilight-jazz",
        Appearance::Dark,
        (
            Rgb::new(0x1C, 0x21, 0x46),
            Rgb::new(0xE9, 0xE4, 0xF2),
            Rgb::new(0xE4, 0xC8, 0x6B),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "noir-vigil",
        Appearance::Dark,
        (
            Rgb::new(0x1D, 0x18, 0x1A),
            Rgb::new(0xE5, 0xDC, 0xDD),
            Rgb::new(0xD5, 0x79, 0x72),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "carnival-ashes",
        Appearance::Dark,
        (
            Rgb::new(0x20, 0x33, 0x2A),
            Rgb::new(0xE3, 0xE5, 0xCF),
            Rgb::new(0xC9, 0xAC, 0x62),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "orbital-frost",
        Appearance::Light,
        (
            Rgb::new(0xEB, 0xF0, 0xF3),
            Rgb::new(0x35, 0x43, 0x4E),
            Rgb::new(0x52, 0x73, 0x8B),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "folded-city",
        Appearance::Dark,
        (
            Rgb::new(0x20, 0x2F, 0x3A),
            Rgb::new(0xDE, 0xE8, 0xEB),
            Rgb::new(0x91, 0xB6, 0xC8),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "midnight-drive",
        Appearance::Dark,
        (
            Rgb::new(0x12, 0x34, 0x3C),
            Rgb::new(0xE7, 0xE6, 0xD3),
            Rgb::new(0xBC, 0xD8, 0xC9),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "editorial-cafe",
        Appearance::Light,
        (
            Rgb::new(0xE5, 0xED, 0xE7),
            Rgb::new(0x36, 0x4A, 0x48),
            Rgb::new(0x4F, 0x7D, 0x75),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "striped-haunt",
        Appearance::Light,
        (
            Rgb::new(0xEF, 0xED, 0xEA),
            Rgb::new(0x33, 0x2B, 0x40),
            Rgb::new(0x77, 0x53, 0x83),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "curious-garden",
        Appearance::Light,
        (
            Rgb::new(0xE7, 0xEE, 0xDD),
            Rgb::new(0x3A, 0x48, 0x40),
            Rgb::new(0x52, 0x7F, 0x70),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "tidebound-gold",
        Appearance::Dark,
        (
            Rgb::new(0x29, 0x2B, 0x23),
            Rgb::new(0xE9, 0xDF, 0xC7),
            Rgb::new(0xD8, 0xB5, 0x6F),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "velvet-cabaret",
        Appearance::Dark,
        (
            Rgb::new(0x35, 0x18, 0x20),
            Rgb::new(0xF1, 0xDE, 0xD5),
            Rgb::new(0xE2, 0xB3, 0x75),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
    ThemeSeed::new(
        "lagoon-whisper",
        Appearance::Dark,
        (
            Rgb::new(0x10, 0x34, 0x37),
            Rgb::new(0xDB, 0xE9, 0xDF),
            Rgb::new(0x8B, 0xC6, 0xBC),
        ),
        (
            "NeNe Nib",
            "MIT; original palette",
            "https://github.com/hideyukiMORI/nene-nib/blob/main/docs/design/2026-10-12-cinema-collection.md",
        ),
    ),
];

/// 名前から。`_` は `-` と同じに受ける。大文字小文字は区別する（名前は小文字だけ）。
#[must_use]
pub fn named(name: &str) -> Option<&'static ThemeSeed> {
    SEEDS.iter().find(|seed| {
        seed.name()
            .chars()
            .eq(name.chars().map(|c| if c == '_' { '-' } else { c }))
    })
}

/// `system` の既定。ダークは `ubuntu-aubergine`、ライトは `neutral-light`（nene-nib と同じ）。
#[must_use]
pub fn default_for(appearance: Appearance) -> &'static ThemeSeed {
    let wanted = SEEDS.iter().find(|seed| seed.appearance() == appearance);
    wanted.unwrap_or(&SEEDS[0])
}

/// 表の中の位置。無ければ `None`。
#[must_use]
pub fn position_of(name: &str) -> Option<usize> {
    SEEDS.iter().position(|seed| seed.name() == name)
}

/// 表の `index` 番目（端で巡る）。
#[must_use]
pub fn at(index: usize) -> &'static ThemeSeed {
    SEEDS.get(index % COUNT).unwrap_or(&SEEDS[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_lowercase_and_findable_with_underscores() {
        for (index, seed) in SEEDS.iter().enumerate() {
            assert!(
                seed.name()
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{}",
                seed.name()
            );
            assert_eq!(position_of(seed.name()), Some(index));
            assert_eq!(
                named(&seed.name().replace('-', "_")).map(ThemeSeed::name),
                Some(seed.name())
            );
            assert!(
                !seed.author().is_empty() && !seed.license().is_empty() && !seed.url().is_empty()
            );
        }
        assert_eq!(named("no-such-theme"), None);
        assert_eq!(default_for(Appearance::Dark).name(), "ubuntu-aubergine");
        assert_eq!(default_for(Appearance::Light).name(), "neutral-light");
        assert_eq!(at(COUNT).name(), SEEDS[0].name());
    }
}
