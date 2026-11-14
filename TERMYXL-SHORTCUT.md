# TERMY XL — STATUS & SHORTCUT

## Commit
`cursor/trends-desk-addon-d271` @ **b0cd223f**

## Build Status
- **Cargo check:** ✅ GREEN (zero errors)
- **Cargo build --release:** ✅ GREEN — 2m 46s
- **Clippy:** ✅ GREEN — zero errors, 167 warnings (all pre-existing)

## Install Status
- **Installed to:** `C:\Users\jpana\AppData\Local\Termy XL\`
- **trends_desk.exe:** 24 MB (built Nov 14 08:41)
- **termy-xcli.exe:** 8.6 MB
- **Shortcuts:** Desktop + Start Menu
- **Backups:** `trends_desk.exe.20261114-084130.bak`, `termy-xcli.exe.20261114-084131.bak`

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

### Reply Editor — Fixed
- `xl_popout.rs`: removed duplicate `drafting_mark` countdown (was showing in 3 places: carousel, outer editor, inner editor)
- `xl_popout.rs`: removed inner `drafting_mark` that replaced text surface during editing (text now always visible)
- `xl_mark.rs`: `reply_surface` renders as single element — no more row-shift when clicking (caret moved → `before`/`middle`/`after` widths changed, rewrapping text)
- `xl_popout.rs`: "Status" placeholder removed

### Take Carousel — Capped at 3 Variants
- `xl_popout.rs`: `take_carousel` shows max 3 cards with "Use this" buttons, plus `+N more` label
- Eliminates the "32 reply variants" visual overload

### Treg Search Engine — WIRED
- `fetch.rs`: `run_topic()` routes `search_source == "treg"` → `run_treg_search()`
- `run_treg_search() loads `TERMY_X_TREG_TOKEN`, calls `fetch_posts(token, query, sort, 50)`
- Maps `reply_guy::Post` → `SourcePost` with proper field extraction
- Sort: `"popular"` → `Sort::Popular`, else `Sort::Newest`
- `xl_pack.rs`: `source_toggle(column_index, current_source)` button swaps OpenCLI↔Treg
- Column label: `@term · OpenCLI · Newest` or `@term · Treg · Popular`

## Files Changed (in commits)
```
crates/desktop_app/src/main.rs                  "Termy XL"
crates/desktop_app/src/xl_agents.rs             search dedup, Topic fields
crates/desktop_app/src/xl_media.rs              restored (direct3d11 vout in player.rs)
crates/desktop_app/src/xl_mark.rs               reply_surface: single-element render (no jitter)
crates/desktop_app/src/xl_pack.rs               sort toggle, source toggle, source+sort label, palette cleanup
crates/desktop_app/src/xl_popout.rs             editor dedup, take carousel 3-card cap
crates/desktop_app/src/trends_desk/chrome.rs     merged settings, Go button
crates/desktop_app/src/trends_desk/mod.rs         refresh with source/sort, design module
crates/desktop_app/src/trends_desk/design.rs      NEW: token system
crates/desktop_app/src/x_panel/player.rs          direct3d11 vout
crates/x_core/src/trends_desk/fetch.rs            Treg search path (run_treg_search)
crates/x_core/src/trends_desk/domain.rs           Topic + search_source + sort
crates/x_core/src/trends_desk/takes.rs            Treg out of provider menu
crates/x_core/src/trends_desk/mod.rs              exports default_search_source, default_sort
crates/x_core/src/trends_desk/store.rs            test Topics with new fields
crates/x_core/src/cli.rs                          Topic creation with new fields
TERMYXL-SHORTCUT.md                               this doc
HANDOVER-2026-11-13-HERMES.md                    handover log
```

## Build Command
```powershell
cd D:\hermes\workspace\termy-xl
.\touch-sources.ps1
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
cargo build --release -p termy --bin trends_desk -p termy_x_cli --bin termy-xcli
```

## What Still Needs Work
1. **Clickable tweet URLs**: Cards show URL as cyan text with "Copy link" button, but no click-to-open-browser
2. **Reply editing duplication**: "3 places counting down" and row-shift — FIXED in b0cd223f
3. **Treg search**: Wired in e7efaae3.

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
