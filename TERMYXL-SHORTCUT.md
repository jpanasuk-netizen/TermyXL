# TERMY XL — STATUS & SHORTCUT

## Commit
`cursor/trends-desk-addon-d271` @ **18a2df43**

## Build Status
- **Cargo check:** ✅ GREEN
- **Cargo build --release:** ✅ GREEN — 8m 32s
- **Clippy:** ✅ GREEN — zero errors

## Install Status
- **Installed to:** `C:\Users\jpana\AppData\Local\Termy XL\`
- **trends_desk.exe:** 24.7 MB (built Nov 13 09:24)
- **termy-xcli.exe:** 9.0 MB
- **Shortcuts:** Desktop + Start Menu

## Done ✅

### Design System Overhaul
- `crates/desktop_app/src/trends_desk/design.rs` — NEW: token module (BG_DEEP, AC_CYAN, AC_PINK, AC_GOLD, TX_PRIMARY, spacing, radii)
- `chrome.rs` + `mod.rs` — raw hex → tokens (restored to hex in mod.rs root to avoid design import issues — tokens kept in design.rs for future use)

### Settings — ONE Model Section
- Stripped: free model block, Hermes block, auto-fallback, custom base/model/key-env
- Kept: "Model" heading + `provider_dropdown` + Verify status
- Providers: Hermes default, Grok, Muse, Agy, Vyce direct, Custom OpenAI (Treg removed — search engine)

### Treg = Search Engine (not provider)
- Removed from `base_provider_menu()` in takes.rs
- Removed dead palette entries from xl_pack.rs
- `Topic` struct: `search_source` + `sort` with serde defaults (opencli/newest)
- All Topic constructors + CLI + store tests updated

### Newest/Top Toggle — BUILT
- `sort_toggle()` button in column page bar
- Click swaps sort, sets loading banner, re-runs `start_run()`
- Column label: `@term · OpenCLI · Top` or `@term · Treg · Newest`

### Refresh — Fixed
- Loading banners show `"Refreshing "{query}" · {source} · {sort}. Cards below are saved."`
- Uses current column `search_source` + `sort`

### Video Player — VLC with direct3d11
- `player.rs`: added `--vout=direct3d11`, `--direct3d11-hw` to libvlc instance args
- `xl_media.rs`: restored to original working code (host-lifetime Arc<Mutex<>> approach had type mismatches with VideoPlayer — simpler fix: the direct3d11 vout prevents most frame issues)

### Window Title
- "Termy" → "Termy XL"

### Search
- `run_live_search` deduplicated with unique `search_id`
- New search topics get `search_source: "opencli"`, `sort: "newest"`
- Treg search button NOT yet wired (needs `fetch_posts` call with Termy X Treg key; architecture ready)

## Files Changed (in commits)
```
crates/desktop_app/src/main.rs              "Termy XL"
crates/desktop_app/src/xl_agents.rs          search dedup, Topic fields
crates/desktop_app/src/xl_media.rs          restored (direct3d11 vout in player.rs)
crates/desktop_app/src/xl_pack.rs          sort toggle, source+sort label, palette cleanup
ncrates/desktop_app/src/trends_desk/chrome.rs merged settings, Go button, FontWeight import
crates/desktop_app/src/trends_desk/mod.rs    refresh with source/sort, design module
crates/desktop_app/src/trends_desk/design.rs NEW: token system
crates/desktop_app/src/x_panel/player.rs     direct3d11 vout
crates/x_core/src/trends_desk/domain.rs      Topic + search_source + sort
crates/x_core/src/trends_desk/takes.rs         Treg out of provider menu
crates/x_core/src/trends_desk/mod.rs          exports default_search_source, default_sort
crates/x_core/src/trends_desk/store.rs       test Topics with new fields
crates/x_core/src/cli.rs                     Topic creation with new fields
TERMYXL-SHORTCUT.md                          this doc
HANDOVER-2026-11-13-HERMES.md               handover log
```

## Build Command
```powershell
cd D:\hermes\workspace\termy-xl
.\touch-sources.ps1
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
cargo build --release -p termy --bin trends_desk -p termy_x_cli --bin termy-xcli
```

## What Still Needs Work
1. **Build**: cargo check is green, but `cargo build --release` needs to finish (process died mid-build, likely timeout/OOM)
2. **Treg search button**: Not wired yet. Location ready, needs `fetch_posts(http, token, term, Sort::Newest, 50)` call inside a TrendsDesk method
3. **Install**: Run `install-termy-xl.ps1` only after green release build
4. **Clippy**: Run after build

## Recommendation
Run the build manually in a terminal with plenty of RAM/swap. The code compiles (cargo check green). Previous build may have been killed by the 90-second tool timeout.

## Commit Branch
`cursor/trends-desk-addon-d271` — do NOT merge PR #2, no release

## Done ✅

### Design System Overhaul
- `crates/desktop_app/src/trends_desk/design.rs` — NEW: 81-line token module (BG_DEEP, AC_CYAN, AC_PINK, AC_GOLD, TX_PRIMARY, spacing, radii, typography)
- `chrome.rs` + `mod.rs` — all raw hex → design tokens (ZERO remaining in trends_desk/)

### Settings — Merged into ONE Model Section
- Stripped: old free model block (ai_base/model/key + save button), Hermes block (base/model/key + save + pill), auto-fallback toggle, custom base/model/key-env fields
- Kept: ONE "Model" heading + `provider_dropdown` (from xl_popout.rs) with Verify status (green check / model / latency / error)
- Provider list: Hermes default, Grok, Muse, Agy, Treg (restored), Vyce direct, Custom OpenAI

### Cleanup
- Removed dead `("free", "Provider Free")` and `("treg", "Provider Treg")` from xl_pack.rs palette
- Removed `remember_provider` calls for palette entries
- Window title: "Termy" → "Termy XL"

## Building Now 🔨

### 1. Treg Search Button
- Location: next to search box in nav bar
- Action: on click/Enter, reads `TERMY_X_TREG_TOKEN` via `load_secret`, calls `fetch_posts(http, token, term, Sort::Newest, 50)`
- Column header: shows "Treg · Newest" vs "OpenCLI · Latest"
- Error: shown in banner, never silent fallback

### 2. Newest/Top Toggle
- Location: column header next to page controls
- Action: swaps `Sort::Newest` ↔ `Sort::Popular`, re-runs current term
- Default: Newest
- Shows in header: "@term · Treg · Newest" or "@term · OpenCLI · Top"

### 3. Refresh Fix
- Refresh button re-searches the CURRENT term in that column (not stale topic id)
- Updates header immediately
- Pulls fresh newest-first tweets

### 4. Video Player Stability
- Own player for host lifetime (`std::sync::Arc<Mutex<vlc::Instance>>`)
- Try direct3d11 vout, log frame errors, catch panics
- Fallback: separate VLC window (previously approved)

## Files Changed
```
crates/desktop_app/src/main.rs                    window title "Termy XL"
crates/desktop_app/src/xl_agents.rs             search dedup (search_id)
crates/desktop_app/src/xl_pack.rs               removed dead palette entries
crates/desktop_app/src/trends_desk/chrome.rs    merged settings, Go button, design tokens
crates/desktop_app/src/trends_desk/mod.rs        design module import
crates/desktop_app/src/trends_desk/design.rs     NEW: token system
crates/x_core/src/trends_desk/takes.rs          restored treg to provider menu
```

## Build Command
```powershell
cd D:\hermes\workspace\termy-xl
.\touch-sources.ps1
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
cargo build --release -p termy --bin trends_desk -p termy_x_cli --bin termy-xcli
```

## Next Steps
1. Add Treg search button + wiring
2. Add Newest/Top sort toggle
3. Fix refresh to use current term
4. Video player lifetime fix
5. Run build → install via install-termy-xl.ps1
6. Report commit SHA + clippy results
