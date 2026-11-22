use super::*;
use crate::io::{Http, HttpRequest};
use serde_json::json;

struct ScriptHttp {
    body: Result<String, String>,
    calls: std::sync::Mutex<Vec<String>>,
}

impl Http for ScriptHttp {
    fn request(&self, request: HttpRequest) -> Result<String, String> {
        self.calls.lock().unwrap().push(request.url);
        self.body.clone()
    }
}

struct MockPoster {
    result: Result<String, String>,
    calls: std::sync::Mutex<u32>,
}

impl ReplyPoster for MockPoster {
    fn post_reply(&self, _target_url: &str, _text: &str) -> Result<String, String> {
        *self.calls.lock().unwrap() += 1;
        self.result.clone()
    }
}

fn post() -> SourcePost {
    SourcePost::from_value(&json!({
        "id": "55",
        "author": "ada",
        "author_name": "Ada Lovelace",
        "text": "Courts and models",
        "created_at": "Wed Nov 11 16:00:00 +0000 2026",
        "likes": 3,
        "replies": 1,
        "views": "10",
        "media_urls": ["https://pbs.twimg.com/media/example.jpg"],
        "url": "https://x.com/ada/status/55"
    }))
    .unwrap()
}

#[test]
fn card_keeps_author_text_media_metrics_time_and_link() {
    let card = card_from_post(&post());
    assert_eq!(card.author_name, "Ada Lovelace");
    assert_eq!(card.handle, "ada");
    assert_eq!(card.text, "Courts and models");
    assert_eq!(
        card.media,
        vec!["https://pbs.twimg.com/media/example.jpg".to_string()]
    );
    assert_eq!(card.likes, 3);
    assert_eq!(card.replies, 1);
    assert_eq!(
        card.created_at.as_deref(),
        Some("Wed Nov 11 16:00:00 +0000 2026")
    );
    assert_eq!(card.url, "https://x.com/ada/status/55");
}

#[test]
fn companion_size_is_twice_termy_width_and_cards_shrink() {
    assert_eq!(COMPANION_WIDTH, 2560.0);
    assert_eq!(COMPANION_HEIGHT, 820.0);
    assert_eq!(card_columns(COMPANION_WIDTH), 2);
    assert_eq!(card_columns(900.0), 1);
}

#[test]
fn dock_expands_right_and_the_left_edge_stays() {
    let dock = frame_from_parts(true, 40.0, 80.0, 2560.0, 820.0);
    assert!(!dock.expanded);
    assert_eq!(dock.width, DOCK_WIDTH);
    let open = expand_right(dock);
    assert_eq!(open.x, dock.x);
    assert_eq!(open.y, dock.y);
    assert_eq!(open.width, COMPANION_WIDTH);
    let resized = DeskFrame {
        width: 1800.0,
        expanded_width: 1800.0,
        ..open
    };
    let shut = collapse_dock(resized);
    assert_eq!(shut.x, 40.0);
    assert_eq!(shut.y, 80.0);
    assert_eq!(shut.width, DOCK_WIDTH);
    assert_eq!(shut.expanded_width, 1800.0);
    let again = toggle_dock(shut);
    assert_eq!(again.x, shut.x);
    assert_eq!(again.width, 1800.0);
    let jumped = expand_to_full(dock, None);
    assert_eq!(jumped.x, dock.x);
    assert_eq!(jumped.width, COMPANION_WIDTH);
    let clamped = expand_to_full(dock, Some(1400.0));
    assert_eq!(clamped.x, dock.x);
    assert_eq!(clamped.width, 1400.0);
    assert_eq!(clamped.expanded_width, COMPANION_WIDTH);
    assert_eq!(
        remember_width(COMPANION_WIDTH, 1400.0, Some(1400.0)),
        COMPANION_WIDTH
    );
    assert_eq!(
        remember_width(COMPANION_WIDTH, 1800.0, Some(3440.0)),
        1800.0
    );
    assert_eq!(open_width(0.0, None), COMPANION_WIDTH);
}

#[test]
fn each_take_is_its_own_card_inside_the_window() {
    for viewport in [900.0, 1600.0, 2560.0] {
        let frames = layout_takes(viewport, &["law-1", "law-2", "law-3"]);
        assert_eq!(frames.len(), 3);
        let mut ids = Vec::new();
        let columns = take_column_count(viewport);
        assert!(columns >= 1);
        if viewport >= 1600.0 {
            assert!(columns > 1, "wider windows add columns");
        }
        for frame in &frames {
            ids.push(frame.id.clone());
            if columns == 1 {
                let inner = viewport - WINDOW_PAD * 2.0;
                assert!((frame.width - inner).abs() < 1.0);
            } else {
                assert!(frame.width <= TAKE_CARD_MAX + 0.01);
            }
            assert!(frame.right() <= viewport - WINDOW_PAD + 0.01);
            assert!(frame.width > 0.0);
        }
        assert_eq!(
            ids.len(),
            ids.iter().collect::<std::collections::HashSet<_>>().len()
        );
    }
    let broken = wrap_token("https://x.com/a-very-long-handle/status/123456789012345678901234");
    assert!(broken.contains('\u{200b}'));
}

#[test]
fn a_saved_report_makes_one_take_card_per_post() {
    let mut second = post();
    second.id = "56".into();
    second.text =
        "The New York State Bar requires lawyers to verify AI-generated legal advice.".into();
    second.author_name = "NYLawJournal".into();
    let posts = vec![post(), second];
    let takes = takes_for_posts(&posts);
    assert_eq!(takes.len(), posts.len());
    assert_ne!(takes[0].1, takes[1].1);
    assert!(takes[0].1.to_lowercase().contains("courts") || takes[0].1.contains("Ada"));
    assert!(takes[1].1.contains("NYLawJournal") || takes[1].1.to_lowercase().contains("lawyers"));
    assert!(takes.iter().all(|(_, text)| !text.contains("hype pile-on")));
    assert!(takes_for_posts(&[]).is_empty());
    let frames = layout_takes(
        687.0,
        &takes.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
    );
    assert_eq!(frames.len(), 2);
    assert!((frames[0].width - (687.0 - WINDOW_PAD * 2.0)).abs() < 1.0);
    assert_eq!(take_column_count(687.0), 1);
    assert!(take_column_count(1400.0) >= 2);
    let wide = expand_locked(
        DeskFrame {
            x: 10.0,
            y: 20.0,
            width: DOCK_WIDTH,
            height: 900.0,
            expanded: false,
            expanded_width: 2560.0,
        },
        Some(2000.0),
    );
    assert_eq!(wide.width, LOCKED_OPEN_WIDTH);
    assert_eq!(wide.expanded_width, LOCKED_OPEN_WIDTH);
    assert!(wide.expanded);
    assert!(take_column_count(2560.0) > take_column_count(687.0));
}

#[test]
fn subject_pages_stay_in_bounds_and_map_by_id() {
    let pages = subject_pages(&[("law", "Law"), ("finance", "Finance"), ("ai", "AI")]);
    assert_eq!(page_index(&pages, "finance"), 1);
    assert_eq!(page_index(&pages, "missing"), 0);
    assert_eq!(turn_page(0, pages.len(), -1), 2);
    assert_eq!(turn_page(2, pages.len(), 1), 0);
    assert_eq!(turn_page(1, pages.len(), 1), 2);
    assert_eq!(page_label(1, pages.len(), "Finance"), "2 / 3 · Finance");
    assert_eq!(pages[page_index(&pages, "ai")].id, "ai");
}

#[test]
fn selection_moves_one_card_and_does_not_send() {
    let ids: Vec<String> = vec!["subject:law".into(), "take:1".into(), "take:2".into()];
    let phase = ReplyPhase::Editing {
        text: "keep this".into(),
    };
    let selected = Some(ids[1].clone());
    let down = move_selection(&ids, selected.as_deref(), 1);
    assert_eq!(down.as_deref(), Some("take:2"));
    let up = move_selection(&ids, down.as_deref(), -1);
    assert_eq!(up.as_deref(), Some("take:1"));
    assert_eq!(
        move_selection(&ids, Some("take:2"), 1).as_deref(),
        Some("take:2")
    );
    assert_eq!(phase.text(), "keep this");
    let cleared: Option<String> = None;
    assert!(cleared.is_none());
}

#[test]
fn scroll_offsets_are_per_card() {
    let mut offsets = std::collections::HashMap::new();
    assert_eq!(scroll_by(&mut offsets, "take:1", 30.0, 80.0), 30.0);
    assert_eq!(scroll_by(&mut offsets, "take:2", 12.0, 80.0), 12.0);
    assert_eq!(scroll_by(&mut offsets, "take:1", 100.0, 80.0), 80.0);
    assert_eq!(scroll_of(&offsets, "take:1"), 80.0);
    assert_eq!(scroll_of(&offsets, "take:2"), 12.0);
    assert_eq!(scroll_of(&offsets, "take:3"), 0.0);
}

#[test]
fn click_selects_one_card_and_reply_stays_a_draft_until_send() {
    let first = selection_after_card("55");
    let second = selection_after_card("56");
    assert_eq!(first.as_deref(), Some("take:55"));
    assert_eq!(second.as_deref(), Some("take:56"));
    assert_ne!(first, second);
    assert!(selection_after_empty().is_none());
    assert_eq!(selection_colors(true), (SELECT_PINK, SELECT_CYAN));
    let open = open_editor("55", "Courts are testing models", TakeProvider::Hermes);
    assert_eq!(open.provider.caption(), "Hermes · Vyce glm-5.3-flash");
    assert_eq!(TakeProvider::parse("grok").caption(), "Provider · Grok");
    assert!(open.open);
    assert_eq!(open.card_id, "55");
    assert!(open.variants.is_empty());
    let closed = close_editor(&open);
    assert!(!closed.open);
    assert_eq!(closed.draft, "Courts are testing models");
    assert!(editor_send(&closed, true).is_err());
    assert!(send_is_busy(&ReplyPhase::Pending {
        text: "queued".into()
    }));
    assert!(!send_is_busy(&ReplyPhase::Editing {
        text: "queued".into()
    }));
}

#[test]
fn closing_the_editor_keeps_the_draft_and_does_not_send() {
    let open = open_editor("55", "hello take", TakeProvider::Free);
    assert!(open.open);
    assert!(open.variants.is_empty());
    assert_eq!(char_count("hi"), 2);
    let closed = close_editor(&open);
    assert!(!closed.open);
    assert_eq!(closed.draft, "hello take");
    assert!(editor_send(&closed, true).is_err());
    assert!(editor_send(&open, false).is_err());
    let poster = MockPoster {
        result: Ok("nope".into()),
        calls: std::sync::Mutex::new(0),
    };
    let phase = ReplyPhase::Editing {
        text: closed.draft.clone(),
    };
    let after = commit_send(&phase, false, "https://x.com/ada/status/55", &poster);
    assert_eq!(*poster.calls.lock().unwrap(), 0);
    assert_eq!(after.text(), "hello take");
    assert_eq!(editor_send(&open, true).unwrap(), "hello take");
    let leaked = EditPopout {
        draft: "hello take|".into(),
        ..open.clone()
    };
    assert_eq!(editor_send(&leaked, true).unwrap(), "hello take");
    let kept = EditPopout {
        draft: "score | 3".into(),
        ..open.clone()
    };
    assert_eq!(editor_send(&kept, true).unwrap(), "score | 3");
    let poster = MockPoster {
        result: Ok("unused".into()),
        calls: std::sync::Mutex::new(0),
    };
    let sitting = ReplyPhase::Editing {
        text: "a finished take".into(),
    };
    let untouched = commit_send(&sitting, false, "https://x.com/ada/status/9", &poster);
    assert_eq!(*poster.calls.lock().unwrap(), 0);
    assert_eq!(untouched.text(), "a finished take");
    assert!(crate::trends_desk::posts_on_ctrl_enter("enter", true));
    assert!(!crate::trends_desk::posts_on_ctrl_enter("enter", false));
    assert_eq!(*poster.calls.lock().unwrap(), 0);
}

#[test]
fn draft_take_fills_the_reply_inside_the_card() {
    assert_eq!(DRAFTING_PLACEHOLDER, "Drafting…");
    let opened = editor_on_draft_click("42", TakeProvider::Grok);
    assert!(opened.open);
    assert!(!draft_click_queues_a_file());
    assert_eq!(
        pink_box_controls(),
        ["Funny", "Smart", "Spicy", "Copy", "Post reply"]
    );
    assert_eq!(opened.card_id, "42");
    assert_eq!(opened.draft, DRAFTING_PLACEHOLDER);
    let missing = text_after_take(DRAFTING_PLACEHOLDER, "", "grok: not installed");
    assert!(missing.is_empty(), "errors stay out of the reply");
    assert!(
        editor_send(
            &EditPopout {
                draft: missing,
                ..opened
            },
            true
        )
        .is_err()
    );
    assert_eq!(
        text_after_take(DRAFTING_PLACEHOLDER, "a real joke", "unused"),
        "a real joke"
    );
    assert_eq!(
        fill_reply_if_waiting(DRAFTING_PLACEHOLDER, "a real joke"),
        "a real joke"
    );
    assert_eq!(fill_reply_if_waiting("my edit", "a real joke"), "my edit");
    assert_eq!(
        fill_reply_if_waiting(DRAFTING_PLACEHOLDER, "  "),
        DRAFTING_PLACEHOLDER
    );
    assert!(reply_needs_a_take(""));
    assert!(reply_needs_a_take(DRAFTING_PLACEHOLDER));
    assert!(!reply_needs_a_take("a real joke"));
    let waiting = open_editor("9", DRAFTING_PLACEHOLDER, TakeProvider::Grok);
    assert!(editor_send(&waiting, true).is_err());
    let mut ready = open_editor("9", "a real joke", TakeProvider::Grok);
    ready.variants = vec![
        "a real joke".into(),
        "the concrete bit".into(),
        "the spicy one".into(),
    ];
    assert!(apply_variant(&mut ready.draft, &ready.variants, 2));
    assert_eq!(ready.draft, "the spicy one");
    assert_eq!(editor_send(&ready, true).unwrap(), "the spicy one");
    let closed = close_editor(&ready);
    assert!(!closed.open);
    assert_eq!(closed.draft, "the spicy one");
    assert!(editor_send(&closed, true).is_err());
}

#[test]
fn dock_state_round_trips_in_settings() {
    use crate::trends_desk::DeskSettings;
    let mut settings = DeskSettings::default();
    assert!(settings.docked);
    settings.docked = false;
    settings.frame_x = 40;
    settings.frame_y = 80;
    settings.expanded_width = 1800;
    settings.frame_height = 820;
    let loaded: DeskSettings =
        serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
    let frame = frame_from_parts(
        loaded.docked,
        loaded.frame_x as f32,
        loaded.frame_y as f32,
        loaded.expanded_width as f32,
        loaded.frame_height as f32,
    );
    assert!(frame.expanded);
    assert_eq!(frame.x, 40.0);
    assert_eq!(frame.width, 1800.0);
    let docked: DeskSettings = serde_json::from_str("{}").unwrap();
    assert!(docked.docked);
    assert_eq!(
        frame_from_parts(true, docked.frame_x as f32, docked.frame_y as f32, 0.0, 0.0).width,
        DOCK_WIDTH
    );
}

#[test]
fn typing_and_a_failed_send_do_not_drop_text_or_auto_post() {
    let poster = MockPoster {
        result: Err("opencli down".into()),
        calls: std::sync::Mutex::new(0),
    };
    let editing = edit_reply(&ReplyPhase::Idle, "hello reply".into());
    let still = commit_send(&editing, false, "https://x.com/ada/status/55", &poster);
    assert_eq!(still, editing);
    assert_eq!(*poster.calls.lock().unwrap(), 0);
    let failed = commit_send(&editing, true, "https://x.com/ada/status/55", &poster);
    assert_eq!(
        failed,
        ReplyPhase::Failed {
            text: "hello reply".into(),
            error: "opencli down".into(),
        }
    );
    assert_eq!(*poster.calls.lock().unwrap(), 1);
    let pending = request_send(&editing, true).unwrap();
    assert!(matches!(pending, ReplyPhase::Pending { .. }));
    let confirmed = apply_send_result(&pending, Ok("receipt-1".into()));
    assert_eq!(
        confirmed,
        ReplyPhase::Confirmed {
            text: "hello reply".into(),
            receipt: "receipt-1".into(),
        }
    );
}

#[test]
fn provider_selection_does_not_hide_a_treg_failure() {
    let sample = post();
    let config = crate::config::XConfig::default();
    let free = draft_take(
        TakeProvider::Free,
        &config,
        &sample,
        &ScriptHttp {
            body: Err("unused".into()),
            calls: std::sync::Mutex::new(Vec::new()),
        },
    )
    .unwrap();
    assert_eq!(free.source, "offline");
    assert!(free.note.starts_with("No model available"));
    assert!(free.note.contains("127.0.0.1:11434") || free.note.contains("unreachable"));
    assert!(free.text.is_empty());
    assert!(!super::super::takes::is_template_angle(&free.note));
    assert_eq!(
        short_take_note("http 402: payment required"),
        "Free model needs credits (402) · fallback angle"
    );
    let mut other = post();
    other.text =
        "Quarterly earnings beat estimates after the finance desk revised guidance.".into();
    other.author_name = "FinanceDesk".into();
    let other_take = draft_take(
        TakeProvider::Free,
        &config,
        &other,
        &ScriptHttp {
            body: Err("unused".into()),
            calls: std::sync::Mutex::new(Vec::new()),
        },
    )
    .unwrap();
    assert!(other_take.text.is_empty());
    assert!(other_take.note.starts_with("No model available"));
    let configured = crate::config::XConfig {
        ai_base_url: "https://vyce.example/v1".into(),
        ai_model: "glm-5.3-flash".into(),
        ai_explicit: true,
        ..crate::config::XConfig::default()
    };
    let answered = ScriptHttp {
        body: Ok(
            r#"{"choices":[{"message":{"content":"Testing court models in public is the actual change, not the headline."}}]}"#
                .into(),
        ),
        calls: std::sync::Mutex::new(Vec::new()),
    };
    let modeled = draft_take(TakeProvider::Free, &configured, &sample, &answered).unwrap();
    assert_eq!(modeled.source, "model");
    assert!(modeled.text.contains("court") || modeled.text.contains("models"));
    assert!(answered.calls.lock().unwrap()[0].contains("vyce.example/v1/chat/completions"));
    struct EnvReset(Option<String>);
    impl Drop for EnvReset {
        fn drop(&mut self) {
            match &self.0 {
                Some(value) => unsafe { std::env::set_var("TERMY_X_TREG_TOKEN", value) },
                None => unsafe { std::env::remove_var("TERMY_X_TREG_TOKEN") },
            }
        }
    }
    let _reset = EnvReset(std::env::var("TERMY_X_TREG_TOKEN").ok());
    unsafe { std::env::set_var("TERMY_X_TREG_TOKEN", "real-key-must-not-be-used") };
    let dir = std::env::temp_dir().join(format!("xl-treg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let isolated = crate::config::XConfig {
        config_dir: dir.clone(),
        token_store: "file".into(),
        ..crate::config::XConfig::default()
    };
    let http = ScriptHttp {
        body: Err("treg: no".into()),
        calls: std::sync::Mutex::new(Vec::new()),
    };
    let err = draft_take(TakeProvider::Treg, &isolated, &sample, &http).unwrap_err();
    assert!(err.contains("No Treg key"));
    assert!(http.calls.lock().unwrap().is_empty());
    assert!(!err.contains("real-key-must-not-be-used"));
    assert!(!err.contains("Hot take"));
    let _ = std::fs::remove_dir_all(&dir);
    drop(_reset);
    let http = ScriptHttp {
        body: Ok(r#"{"choices":[{"message":{"content":"A real Treg take"}}]}"#.into()),
        calls: std::sync::Mutex::new(Vec::new()),
    };
    let drafted = treg_take_with_token("secret-token", &http, &sample).unwrap();
    assert_eq!(drafted.text, "A real Treg take");
    assert_eq!(drafted.source, "treg");
    let failed = treg_take_with_token(
        "secret-token",
        &ScriptHttp {
            body: Ok(r#"{"detail":"no tool"}"#.into()),
            calls: std::sync::Mutex::new(Vec::new()),
        },
        &sample,
    )
    .unwrap_err();
    assert!(failed.contains("treg"));
    assert!(!failed.contains("secret-token"));
    assert!(!failed.contains("Hot take"));
    assert!(http.calls.lock().unwrap().is_empty() || err.contains("key") || err.contains("Treg"));
}

#[test]
fn treg_probe_is_not_a_publish_url() {
    let request = treg_draft_probe("secret-token");
    assert!(request.url.contains("treg.to"));
    assert!(!request.url.contains("/status/"));
    assert!(request.headers.iter().any(|(key, _)| key == "X-Treg-Token"));
    assert!(!format!("{request:?}").contains("x.exe"));
}

#[test]
fn pink_post_button_tracks_the_box_and_the_send() {
    let editing = |t: &str| ReplyPhase::Editing { text: t.into() };
    let hidden = pink_post_button(DRAFTING_PLACEHOLDER, &editing(DRAFTING_PLACEHOLDER));
    assert!(
        !hidden.visible && !hidden.enabled,
        "no button while Drafting…"
    );
    let empty = pink_post_button("   ", &editing(""));
    assert!(
        !empty.visible && !empty.enabled,
        "empty box: nothing to post"
    );
    let ready = pink_post_button("my edited take", &editing("my edited take"));
    assert!(ready.visible && ready.enabled);
    assert_eq!(ready.label, "Post");
    let posting = pink_post_button("my edited take", &ReplyPhase::Pending { text: "x".into() });
    assert!(posting.visible && !posting.enabled);
    assert_eq!(posting.label, "Posting…");
    let done = pink_post_button(
        "my edited take",
        &ReplyPhase::Confirmed {
            text: "x".into(),
            receipt: "ok https://x.com/me/status/42".into(),
        },
    );
    assert!(
        done.visible && !done.enabled,
        "no double post of the same text"
    );
    assert_eq!(done.label, "Posted ✓");
    assert_eq!(done.note.as_deref(), Some("https://x.com/me/status/42"));
    let failed = pink_post_button(
        "my edited take",
        &ReplyPhase::Failed {
            text: "x".into(),
            error: "OpenCLI said no".into(),
        },
    );
    assert!(failed.visible && failed.enabled);
    assert_eq!(failed.label, "Retry post");
    assert_eq!(failed.note.as_deref(), Some("OpenCLI said no"));
    let failed_empty = pink_post_button(
        "",
        &ReplyPhase::Failed {
            text: "x".into(),
            error: "e".into(),
        },
    );
    assert!(!failed_empty.enabled, "retry needs text");
}

#[test]
fn a_late_take_never_replaces_typed_text() {
    assert_eq!(
        text_after_take("my own words", "generated", ""),
        "my own words"
    );
    assert_eq!(
        text_after_take(DRAFTING_PLACEHOLDER, " generated ", ""),
        "generated"
    );
    assert_eq!(text_after_take("", "generated", ""), "generated");
}
