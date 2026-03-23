use egui::Color32;

/// All visual constants in one place. Change here to restyle the whole app.
pub struct Theme;

impl Theme {
    // ── Background ────────────────────────────────────────────────────────────
    pub const BG_PRIMARY:   Color32 = Color32::from_rgb(15,  23,  42);  // slate-900
    pub const BG_PANEL:     Color32 = Color32::from_rgb(30,  41,  59);  // slate-800
    pub const BG_CARD:      Color32 = Color32::from_rgb(51,  65,  85);  // slate-700
    pub const BG_HOVER:     Color32 = Color32::from_rgb(71,  85, 105);  // slate-600
    pub const BG_SELECTED:  Color32 = Color32::from_rgb(30,  58, 138);  // blue-900

    // ── Text ──────────────────────────────────────────────────────────────────
    pub const TEXT_PRIMARY:   Color32 = Color32::from_rgb(248, 250, 252); // slate-50
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // slate-400
    pub const TEXT_MUTED:     Color32 = Color32::from_rgb(100, 116, 139); // slate-500
    pub const TEXT_ACCENT:    Color32 = Color32::from_rgb( 96, 165, 250); // blue-400
    pub const TEXT_SUCCESS:   Color32 = Color32::from_rgb( 52, 211, 153); // emerald-400
    pub const TEXT_WARNING:   Color32 = Color32::from_rgb(251, 191,  36); // amber-400
    pub const TEXT_ERROR:     Color32 = Color32::from_rgb(248, 113, 113); // red-400

    // ── Accents ───────────────────────────────────────────────────────────────
    pub const ACCENT_BLUE:     Color32 = Color32::from_rgb( 59, 130, 246); // blue-500
    pub const ACCENT_GREEN:    Color32 = Color32::from_rgb( 16, 185, 129); // emerald-500
    pub const ACCENT_ORANGE:   Color32 = Color32::from_rgb(249, 115,  22); // orange-500

    // ── Graph nodes ───────────────────────────────────────────────────────────
    pub const NODE_INITIAL:   Color32 = Color32::from_rgb( 16, 185, 129); // green
    pub const NODE_GOAL:      Color32 = Color32::from_rgb( 59, 130, 246); // blue
    pub const NODE_NORMAL:    Color32 = Color32::from_rgb( 71,  85, 105); // slate
    pub const NODE_BORDER:    Color32 = Color32::WHITE;
    pub const EDGE_COLOR:     Color32 = Color32::from_rgb(100, 116, 139); // slate-500

    // ── Bit layout ────────────────────────────────────────────────────────────
    pub const UNUSED_BIT_COLOR: Color32 = Color32::from_gray(40);
    pub const BITMASK_BIT_ON:    Color32 = Color32::from_rgb(0, 150, 255);
    pub const BIT_COLORS: [Color32; 16] = [
        Color32::from_rgb( 59, 130, 246), // 0: Blue
        Color32::from_rgb(239,  68,  68), // 1: Red
        Color32::from_rgb( 34, 197,  94), // 2: Green
        Color32::from_rgb(168,  85, 247), // 3: Violet
        Color32::from_rgb(249, 115,  22), // 4: Orange
        Color32::from_rgb(  6, 182, 212), // 5: Cyan
        Color32::from_rgb(236,  72, 153), // 6: Pink
        Color32::from_rgb(234, 179,   8), // 7: Yellow
        Color32::from_rgb( 99, 102, 241), // 8: Indigo
        Color32::from_rgb(244,  63,  94), // 9: Rose
        Color32::from_rgb( 20, 184, 166), // 10: Teal
        Color32::from_rgb(132, 204,  22), // 11: Lime
        Color32::from_rgb(139,  92, 246), // 12: Purple
        Color32::from_rgb( 14, 165, 233), // 13: Sky
        Color32::from_rgb(217,  70, 239), // 14: Fuchsia
        Color32::from_rgb(245, 158,  11), // 15: Amber
    ];

    pub fn bit_color(field_index: usize) -> Color32 {
        // This will now cycle through 16 colors safely
        Self::BIT_COLORS[field_index % Self::BIT_COLORS.len()]
    }

    // ── Sizes ─────────────────────────────────────────────────────────────────
    pub const PANEL_ROUNDING: f32  = 6.0;
    pub const CARD_ROUNDING:  f32  = 4.0;
    pub const BUTTON_HEIGHT:  f32  = 26.0;
    pub const ROW_HEIGHT:     f32  = 22.0;
    pub const ICON_SIZE:      f32  = 16.0;
    pub const SEPARATOR_W:    f32  = 1.0;
}
