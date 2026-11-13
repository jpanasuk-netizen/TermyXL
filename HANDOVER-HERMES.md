# Leo-Bot handover to Hermes (glm-5.3-flash), 2026-11-13 ~5:45 AM CT

Jeremy asked that Hermes take over the Termy XL work and keep notes on everything Leo-Bot had open.
These rules from Jeremy always apply: nothing gets sent, spent, published, posted, or merged without his explicit OK. Never invent numbers ("no source, no number"). Never log or repeat keys. Public contact info is jpanasuk@gmail.com and New Auburn, WI.

---------------------------------------------------------------
## 1. TERMY XL (active, top priority)

**Where things are**
- Code (yours to edit): D:\hermes\workspace\termy-xl  (WSL: /mnt/d/hermes/workspace/termy-xl)
  - Repo https://github.com/jpanasuk-netizen/termy-x, branch cursor/trends-desk-addon-d271, PR #2 https://github.com/jpanasuk-netizen/termy-x/pull/2
  - HEAD 18eae7e. Run `git pull` first, because the cloud agent was cancelled mid-run at about 5:44 AM CT and may have pushed a partial commit. Review anything newer than 18eae7e before you build on it.
- Installed app: %LOCALAPPDATA%\Termy XL\  (trends_desk.exe, termy-xcli.exe). Running PID 42872 since 11/12 2:42 PM CT.
  - trends_desk SHA256 18BE82C49A2BDA95E08457642BDFA14FA1D7E4D63A77076174AC60273DC9737B
- Data and log: %APPDATA%\Termy XL\  (logs\termy-xl.log, grok-models.txt cache)
- Older handoff: D:\hermes\Termy-X-Handoff-2026-11-08 (this covers Termy X, the separate original app)

**Never touch**
- Termy X (the original app), %APPDATA%\termy, its shortcuts, x-drafts.json, and window-state.json
- Never run x.exe on LightBringer. Use termy-xcli.exe.
- No paid official X API
- Never click Post, Send, Like, Repost, Delete, or "Use this" while testing, and make no test posts without Jeremy's OK
- Don't edit the Hermes config.yaml or restart Hermes without his go, and don't change Agy auth without asking
- Don't type in his live window. Close the app with WM_CLOSE only.

**Windows build (only install after a green build)**
1. touch-sources.ps1
2. vcvars64 (rustc 1.98.1). Set `RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc` for that session only.
3. cargo build --release -p termy --bin trends_desk -p termy_x_cli --bin termy-xcli
4. install-termy-xl.ps1
Wait until the app has been idle for 2 minutes before replacing it, unless Jeremy says to do it now.
Run the tests, clippy, and the architecture/file-size check before pushing. trends_desk is already split across mod.rs, chrome.rs, and xl_*.rs to stay under the size limit.

**Standing behavior Jeremy wants (keep all of it)**
- Hermes (Vyce glm-5.3-flash) is the default take provider. Other providers: Grok, Muse, Agy, Treg, Vyce direct, Custom OpenAI endpoint.
- No automatic fallback, ever. If a provider fails, show the exact error plus buttons for "Try Hermes" and "Try Muse".
- Grok gets up to 5 minutes, with a breathing "…" plus a seconds counter and a Cancel button. Never pass --model unless one is picked. The CLI default is grok-4.7. The model list comes from `grok models` (hidden, 20 s timeout, 1 h cache). Never call `grok --help`.
- Agy 1.5.7 uses Jeremy's Google login. modelProvider was removed from ~/.gemini/antigravity-cli/settings.json (backup settings.json.bak-2026-11-12). It verifies green.
- A take is made only when he clicks Draft take. Never in the background, and no visible Grok windows.
- Takes show in a carousel in the pink box. A new take is made only when he arrows past the last card.
- The pink box text is fully editable, and errors stay out of the reply text. No stray "|" at the end.
- Every take is saved to Drafts ("Saved to Drafts ✓"). Drafts has Copy, Copy all, Post reply (with the real @handle), and Delete, and shows a check after a confirmed post.
- A posted card closes its pink box and gets a thick bright-gold border.
- Provider Verify makes a real test call: a green check plus model and ms, or the exact error. His pick is remembered.
- 687 locked width, single instance. He uses an ultrawide (3440 px).

**Open fixes Jeremy asked for (11/13 morning)**
1. Treg search button is gone. Commit 1193cdba removed the "Treg takes" button, and desk search (crates/desktop_app/src/xl_agents.rs run_live_search, then OpenCliDesk.run_topic) is OpenCLI-only. Add a "Search via Treg" option next to the search box. It should use the saved Termy X Treg key read-only (x_core reply_guy.rs and rg::load_secret), run only on his click or Enter, show the source, and show the real error with no silent fallback. Also put Treg back in the provider dropdown.
2. Settings have about 3 separate sections for the model. Merge them into ONE "Model" section: a provider dropdown, a model dropdown for that provider, and a Verify button with result. Remember the selection. Related code: chrome.rs ("Default take provider", around line 724), the provider_label around line 243, and the xl_pack.rs palette entries "Provider Free/Treg".
3. Search returns aggregated digests instead of real tweets. Fix it to return real individual posts, newest first: @handle, time, text, and link. Add a Newest/Top toggle (Newest default), show source and sort in the column header, and don't pad with old or cached posts. Add a test that Newest results are strictly in time-descending order.
4. Refresh uses a stale term. After he types a new term and clicks Refresh, the header shows the old term and he gets nothing. Every search column uses the id "search", and mod.rs:386 says "cards stay on the saved report". Fix: use the column's current term, update the header right away, show a loading state, and fetch live. Give each column its own id keyed by the term, and drop stale async results.
5. Video plays as a black box, sometimes with audio, isn't smooth, and then crashes. The log shows only "INFO media: video host shown". libvlc decodes, but frames never reach the HWND, and the player is probably freed mid-play. Make it seamless and crash-free: own the player for the host's lifetime, try the direct3d11 vout, and fall back to a native separate VLC window with native controls (Jeremy approved this earlier). Keep in-app photo viewing, log frame and errors, and catch panics.

**Things seen in the log 11/13 morning:** Hermes at 127.0.0.1:8642 had network errors at times, Vyce gave 503 "model currently…" errors, and the free Ollama model glm-5.2:cloud returned 402 (needs paid credits). A stale `grok-4-fast-non-reasoning` error came from an old build.

**Also pending on Termy XL**
- CI on 18eae7e: architecture and clippy pass. The 2 paired "facade throughput" bench jobs fail with "can't find crate for termy_core" in terminal_facade_bench.rs. That's unrelated to the desk, but it's worth fixing.
- Merging PR #2 needs Jeremy's OK.
- An unexplained post went out on 11/12 at 1:25:20 PM CT during an audit: https://x.com/jasper_Black/status/2120961165526270150 (parent 2119163332234301596). Jeremy never confirmed whether it was him. Ask before assuming.
- Dock test with Termy X open is still to do.
- The cloud agent bc-7b900526-8622-5fdc-98b7-a3e2e334d271 is cancelled. Leo-Bot can restart it if Hermes wants help.

---------------------------------------------------------------
## 2. Money and cash war (MoneyBot owns cash; BTCC owns signals)
- PNC x8895 is in Low Cash: $2.95 as of 11/12 6:00 AM CT.
- X moved Jeremy from Ultra to Premium and charged $30.00 on 11/12 at 4:49 PM CT (card ending 8273). The next $30 charge is Dec 12 unless he cancels by Dec 11.
- The Kraken USD withdrawals from 11/8 and 11/11 (after the XRP sale) still show initiated only, with no completed or rejected notice.
- No GoFundMe, PayPal, or Kalshi bank clears seen through 11/12.
- 19-day cash war board: lane A MoneyBot free-money, lane B CONNECT drafts (drafts only), lane C Ret plus MR content and Shorts.

## 3. Email and people
- Savage Labs seed fund: Jeremy's reply to Cameron Pierre (cameron@savage.ventures) went out 11/12 at 2:35 PM CT with the deck https://qstreamingllc.com/deck/. No reply yet.
- email-relay PR #1: leave the draft alone. Don't merge it or set SMTP_PASSWORD until the cron is manual-only.

## 4. Content and publishing
- MR (The Machine Room) publish clockwork runs weekdays at 5:45 PM and was DRY on 11/12. There have been no new MiMo renders since B2_SVB (9/16). Batch2 (FTX, Hollywood, credit cards) has only scripts and thumbnails. AutoSocial :3006 is down and :3000 is up (Kalshi proxy). Receipts go in HermesVault/50-YouTube/machine-room/work/.
- Pending: apply the V2/V3 custom thumbnails in YouTube Studio, the Muse.ai MR video pipeline (5 videos from the 0924 topics, no publishing), and X growth follow-ups with the new YouTube link.
- CONNECT companion alignment: drafts are in /workspace/connect-companion-alignment-2026-11-10/ on the GrokBot box. Waiting on Jeremy's approval for each destination via BTCC: the MeshKore edit route, and whether LinkedIn edits the existing post or gets a new one.
- Hugging Face write token needed if Jeremy wants a deck copy on HF.

## 5. Other in-flight work (other bots own these; status only)
- Grok Build: ultimate quant site plus monorepo (paper trading only, 3440 px first)
- Claude Code on LightBringer: DaviddTech hour-of-day BTC backtest (paper only)
- Hermes Bot: fix New Chat so it condenses the thread and stops bouncing to the old session
- Termy X release package: bundle, checksums, no secrets. No public upload without approval.

## 6. Routines Leo-Bot runs (on GrokBot)
Path E morning CoS brief (8:00 weekdays), Path E morning review (9:30), MR IG morning publish check (10:45), Path E evening review (5:30 PM), MR publish clockwork (5:45 PM), Fri OpsLog rollup (4 PM), Polymarket repos git pull (8:00), Morning love quote poster (7:17 daily), Cool new money bot (Mon 9:00, last run failed). Once GrokBot access is gone, these need Hermes cron equivalents.
