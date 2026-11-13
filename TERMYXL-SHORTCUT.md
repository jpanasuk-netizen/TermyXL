# TERMY XL — STATUS & SHORTCUT

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
