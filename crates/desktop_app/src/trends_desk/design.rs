//! Design tokens for Termy XL — one source of truth for colors, spacing,
//! typography, and animation timing. Imported by chrome.rs and mod.rs.

// ─── Color Palette ─────────────────────────────────────────────────────────
/// Near-black with warm undertone — 1px above pure black.
pub const BG_DEEP: u32 = 0x070b14;
/// Header, nav bar — slightly elevated.
pub const BG_NAV: u32 = 0x0d1117;
/// Card surface — warm charcoal.
pub const BG_CARD: u32 = 0x131820;
/// Elevated card / hover state.
pub const BG_CARD_HOVER: u32 = 0x1a2230;
/// Button rest surface.
pub const BG_BUTTON: u32 = 0x1c2332;
/// Active / selected surface.
pub const BG_ACTIVE: u32 = 0x1f2d42;
/// Input / editable field background.
pub const BG_INPUT: u32 = 0x101820;
/// Danger / destructive surface (rose hint).
pub const BG_DANGER: u32 = 0x2a1018;

/// Primary accent — electric cyan.
pub const AC_CYAN: u32 = 0x7af6ff;
/// Secondary accent — neon pink.
pub const AC_PINK: u32 = 0xff4fa3;
/// Tertiary accent — warm gold.
pub const AC_GOLD: u32 = 0xffe56a;
/// Success — emerald.
pub const AC_OK: u32 = 0x34d399;
/// Error — rose.
pub const AC_ERROR: u32 = 0xf43f5e;

/// Border default — steel blue at 15% opacity (visualized as solid).
pub const BD_DEFAULT: u32 = 0x1a2535;
/// Border hover / focus — brighter.
pub const BD_FOCUS: u32 = 0x2a3a50;
/// Border accent — cyan glow.
pub const BD_ACCENT: u32 = 0x4af0ff;
/// Border danger.
pub const BD_DANGER: u32 = 0xf43f5e;

/// Text primary — warm white.
pub const TX_PRIMARY: u32 = 0xf0f0f5;
/// Text secondary — cool gray.
pub const TX_SECONDARY: u32 = 0x8b95a5;
/// Text muted — steel.
pub const TX_MUTED: u32 = 0x5a6678;
/// Text inverse — on accent backgrounds.
pub const TX_INVERSE: u32 = 0x070b14;

// ─── Typography ────────────────────────────────────────────────────────────
pub const TEXT_XS: f32 = 11.0;
pub const TEXT_SM: f32 = 13.0;
pub const TEXT_BASE: f32 = 14.0;
pub const TEXT_LG: f32 = 16.0;
pub const TEXT_XL: f32 = 20.0;

// ─── Spacing / Geometry ────────────────────────────────────────────────────
pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 10.0;
pub const RADIUS_LG: f32 = 14.0;
pub const RADIUS_XL: f32 = 20.0;

pub const GAP_XS: f32 = 4.0;
pub const GAP_SM: f32 = 8.0;
pub const GAP_MD: f32 = 12.0;
pub const GAP_LG: f32 = 16.0;
pub const GAP_XL: f32 = 24.0;

pub const PAD_SM: f32 = 6.0;
pub const PAD_MD: f32 = 10.0;
pub const PAD_LG: f32 = 16.0;

/// Minimum touch target (button height).
pub const BTN_MIN_H: f32 = 36.0;

// ─── Animation / Timing ────────────────────────────────────────────────────
pub const DUR_FAST: u64 = 100;   // hover states
pub const DUR_BASE: u64 = 150; // transitions
pub const DUR_SLOW: u64 = 250;   // panel opens
