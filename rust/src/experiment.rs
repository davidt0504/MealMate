//! The local wording experiment (OPT-007 §10): which label the week action wears, and a
//! tester-only, on-device record of how sessions went. Its state lives in one small versioned
//! file in the app-support directory — never in the household database — so it survives
//! restarts and imports, travels in no export of household data, and reaches nobody unless a
//! tester exports it by hand. Nothing here is an input to planning: the label is presentation.
//!
//! Every stored field is on the allowlist by construction: the event struct has exactly the
//! fields §10 permits and `deny_unknown_fields` on read. No household or recipe id, title,
//! ingredient or restriction text, free text or meal date can be written, because no field can
//! hold one — the session id is minted here, not accepted from the caller.

use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub const SCHEMA: u32 = 1;
pub const EXPERIMENT_ID: &str = "week-action-label-v1";
pub const MAX_EVENTS: usize = 10_000;
pub const FILE_NAME: &str = "wording_experiment.json";

/// Stable identifiers first; displayed words may change without changing an id.
pub const LABELS: [(&str, &str); 10] = [
    ("new_mix", "New mix"),
    ("refresh", "Refresh"),
    ("another_plan", "Another plan"),
    ("remix", "Remix"),
    ("change_meals", "Change meals"),
    ("new_suggestions", "New suggestions"),
    ("mix_it_up", "Mix it up"),
    ("replan", "Replan"),
    ("different_meals", "Different meals"),
    ("try_another", "Try another"),
];
pub const DEFAULT_LABEL: &str = "new_mix";
/// The two arms, assigned 1:1.
pub const VARIANTS: [&str; 2] = ["new_mix", "another_plan"];

pub fn label_text(id: &str) -> Option<&'static str> {
    LABELS.iter().find(|(i, _)| *i == id).map(|(_, t)| *t)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// The enabled week action was rendered; once per session.
    Exposure,
    AlternativeRequested,
    AlternativeResult,
    Undo,
    /// Once per session: a retried Accept is not a second acceptance.
    AcceptSuccess,
    Discard,
    Background,
    Resume,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub schema: u32,
    pub experiment: String,
    pub variant: String,
    pub session: String,
    pub kind: EventKind,
    /// Active foreground time since the session began, from a monotonic clock.
    pub elapsed_ms: u64,
    pub changed: u32,
    pub exhausted: u32,
    pub overridden: bool,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    id: String,
    variant: String,
    active: bool,
    overridden: bool,
    exposed: bool,
    accepted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema: u32,
    enabled: bool,
    /// Tester changes wait for the next session, so a label never changes mid-review.
    next_enabled: Option<bool>,
    assigned: Option<String>,
    override_label: Option<String>,
    next_override: Option<Option<String>>,
    session: Option<Session>,
    events: Vec<Event>,
    dropped: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub session_id: String,
    pub label_id: String,
    pub label: String,
    /// Recording: the experiment is enabled for this session.
    pub active: bool,
    pub overridden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub enabled: bool,
    pub next_enabled: Option<bool>,
    pub assigned: Option<String>,
    pub override_label: Option<String>,
    pub next_override: Option<Option<String>>,
    pub events: u32,
    pub dropped: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    pub schema: u32,
    pub experiment: String,
    pub truncated: bool,
    pub dropped: u64,
    pub events: Vec<Event>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExperimentError {
    #[error("experiment file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown label {0:?}")]
    UnknownLabel(String),
}

/// One writer at a time across FRB's worker threads.
static FILE_LOCK: Mutex<()> = Mutex::new(());

fn load(dir: &Path) -> State {
    // Tester telemetry: an unreadable or foreign-version file starts over rather than failing
    // the planning screen that asked for a label.
    std::fs::read_to_string(dir.join(FILE_NAME))
        .ok()
        .and_then(|text| serde_json::from_str::<State>(&text).ok())
        .filter(|s| s.schema == SCHEMA)
        .unwrap_or(State {
            schema: SCHEMA,
            ..State::default()
        })
}

fn save(dir: &Path, state: &State) -> Result<(), ExperimentError> {
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{FILE_NAME}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec(state).expect("plain data"))?;
    std::fs::rename(tmp, dir.join(FILE_NAME))?;
    Ok(())
}

fn with_state<T>(
    dir: &Path,
    f: impl FnOnce(&mut State) -> Result<T, ExperimentError>,
) -> Result<T, ExperimentError> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut state = load(dir);
    let out = f(&mut state)?;
    save(dir, &state)?;
    Ok(out)
}

fn hex(bytes: &[u8; 16]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Begins one planning session: applies pending tester changes, assigns the variant once per
/// experiment version if enabled, and fixes the label for the whole session. `nonce` is fresh
/// randomness from the caller; the assignment reads one bit of it.
pub fn start_session(dir: &Path, nonce: [u8; 16]) -> Result<SessionInfo, ExperimentError> {
    with_state(dir, |s| {
        if let Some(enabled) = s.next_enabled.take() {
            s.enabled = enabled;
        }
        if let Some(label) = s.next_override.take() {
            s.override_label = label;
        }
        if s.enabled && s.assigned.is_none() {
            s.assigned = Some(VARIANTS[usize::from(nonce[0] & 1)].to_owned());
        }
        let overridden = s.enabled && s.override_label.is_some();
        let variant = match (&s.override_label, &s.assigned) {
            (Some(label), _) if s.enabled => label.clone(),
            (_, Some(assigned)) if s.enabled => assigned.clone(),
            _ => DEFAULT_LABEL.to_owned(),
        };
        let session = Session {
            id: hex(&nonce),
            variant: variant.clone(),
            active: s.enabled,
            overridden,
            exposed: false,
            accepted: false,
        };
        s.session = Some(session.clone());
        Ok(SessionInfo {
            session_id: session.id,
            label: label_text(&variant).unwrap_or("New mix").to_owned(),
            label_id: variant,
            active: s.enabled,
            overridden,
        })
    })
}

/// Records one event of the current session. Returns whether it was kept: an event from any
/// other session, from an inactive one, a second exposure or a second acceptance is dropped.
pub fn record(
    dir: &Path,
    session_id: &str,
    kind: EventKind,
    elapsed_ms: u64,
    changed: u32,
    exhausted: u32,
    completed: bool,
) -> Result<bool, ExperimentError> {
    with_state(dir, |s| {
        let Some(session) = s
            .session
            .as_mut()
            .filter(|x| x.id == session_id && x.active)
        else {
            return Ok(false);
        };
        match kind {
            EventKind::Exposure if session.exposed => return Ok(false),
            EventKind::Exposure => session.exposed = true,
            EventKind::AcceptSuccess if session.accepted => return Ok(false),
            EventKind::AcceptSuccess => session.accepted = true,
            _ => {}
        }
        let event = Event {
            schema: SCHEMA,
            experiment: EXPERIMENT_ID.to_owned(),
            variant: session.variant.clone(),
            session: session.id.clone(),
            kind,
            elapsed_ms,
            changed,
            exhausted,
            overridden: session.overridden,
            completed,
        };
        s.events.push(event);
        if s.events.len() > MAX_EVENTS {
            let over = s.events.len() - MAX_EVENTS;
            s.events.drain(..over);
            s.dropped += over as u64;
        }
        Ok(true)
    })
}

pub fn set_enabled(dir: &Path, enabled: bool) -> Result<(), ExperimentError> {
    with_state(dir, |s| {
        s.next_enabled = Some(enabled);
        Ok(())
    })
}

pub fn set_override(dir: &Path, label: Option<String>) -> Result<(), ExperimentError> {
    if let Some(id) = &label {
        if label_text(id).is_none() {
            return Err(ExperimentError::UnknownLabel(id.clone()));
        }
    }
    with_state(dir, |s| {
        s.next_override = Some(label);
        Ok(())
    })
}

/// Clears the assignment, overrides and every recorded event; the enabled flag stays. The next
/// session assigns afresh.
pub fn reset(dir: &Path) -> Result<(), ExperimentError> {
    with_state(dir, |s| {
        *s = State {
            schema: SCHEMA,
            enabled: s.enabled,
            next_enabled: s.next_enabled,
            ..State::default()
        };
        Ok(())
    })
}

pub fn status(dir: &Path) -> Status {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let s = load(dir);
    Status {
        enabled: s.enabled,
        next_enabled: s.next_enabled,
        assigned: s.assigned,
        override_label: s.override_label,
        next_override: s.next_override,
        events: s.events.len() as u32,
        dropped: s.dropped,
    }
}

/// Writes the sanitized record to `dest`, on explicit tester action only. Returns the event
/// count.
pub fn export(dir: &Path, dest: &Path) -> Result<u32, ExperimentError> {
    let _guard = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let s = load(dir);
    let out = Export {
        schema: SCHEMA,
        experiment: EXPERIMENT_ID.to_owned(),
        truncated: s.dropped > 0,
        dropped: s.dropped,
        events: s.events,
    };
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, serde_json::to_vec_pretty(&out).expect("plain data"))?;
    Ok(out.events.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nonce(n: u8) -> [u8; 16] {
        [n; 16]
    }

    fn enabled_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        set_enabled(dir.path(), true).unwrap();
        dir
    }

    #[test]
    fn disabled_by_default_the_label_is_new_mix_and_nothing_records() {
        let dir = tempfile::tempdir().unwrap();
        let s = start_session(dir.path(), nonce(1)).unwrap();
        assert_eq!(s.label, "New mix");
        assert!(!s.active);
        assert!(!record(
            dir.path(),
            &s.session_id,
            EventKind::Exposure,
            0,
            0,
            0,
            false
        )
        .unwrap());
        assert_eq!(status(dir.path()).events, 0);
    }

    #[test]
    fn the_variant_is_assigned_once_and_kept_across_sessions() {
        let dir = enabled_dir();
        let first = start_session(dir.path(), nonce(1)).unwrap();
        assert_eq!(first.label_id, "another_plan");
        for n in [0, 2, 4] {
            let again = start_session(dir.path(), nonce(n)).unwrap();
            assert_eq!(again.label_id, "another_plan", "never reassigned");
        }
        let other = enabled_dir();
        assert_eq!(
            start_session(other.path(), nonce(2)).unwrap().label_id,
            "new_mix"
        );
    }

    #[test]
    fn changes_take_effect_next_session_not_mid_review() {
        let dir = enabled_dir();
        let s = start_session(dir.path(), nonce(0)).unwrap();
        set_override(dir.path(), Some("remix".to_owned())).unwrap();
        set_enabled(dir.path(), false).unwrap();
        // The running session keeps recording under its own label.
        assert!(record(
            dir.path(),
            &s.session_id,
            EventKind::Exposure,
            5,
            0,
            0,
            false
        )
        .unwrap());
        let next = start_session(dir.path(), nonce(3)).unwrap();
        assert!(!next.active);
        assert_eq!(next.label, "New mix");
        set_enabled(dir.path(), true).unwrap();
        let later = start_session(dir.path(), nonce(4)).unwrap();
        assert_eq!(later.label, "Remix");
        assert!(later.overridden);
    }

    #[test]
    fn exposure_and_acceptance_count_once_and_other_sessions_are_ignored() {
        let dir = enabled_dir();
        let s = start_session(dir.path(), nonce(0)).unwrap();
        let p = dir.path();
        assert!(record(p, &s.session_id, EventKind::Exposure, 10, 0, 0, false).unwrap());
        assert!(!record(p, &s.session_id, EventKind::Exposure, 20, 0, 0, false).unwrap());
        assert!(record(p, &s.session_id, EventKind::AcceptSuccess, 30, 0, 0, true).unwrap());
        assert!(!record(p, &s.session_id, EventKind::AcceptSuccess, 31, 0, 0, true).unwrap());
        assert!(!record(p, "someone-else", EventKind::Undo, 1, 0, 0, false).unwrap());
        assert_eq!(status(p).events, 2);
    }

    #[test]
    fn a_session_that_never_ends_stays_incomplete() {
        let dir = enabled_dir();
        let p = dir.path();
        let killed = start_session(p, nonce(0)).unwrap();
        record(p, &killed.session_id, EventKind::Exposure, 10, 0, 0, false).unwrap();
        record(
            p,
            &killed.session_id,
            EventKind::Background,
            40,
            0,
            0,
            false,
        )
        .unwrap();
        // The process died here; the next launch starts a new session.
        let next = start_session(p, nonce(2)).unwrap();
        record(p, &next.session_id, EventKind::End, 5, 0, 0, true).unwrap();
        let out = dir.path().join("out.json");
        export(p, &out).unwrap();
        let exported: Export = serde_json::from_slice(&std::fs::read(out).unwrap()).unwrap();
        assert!(!exported
            .events
            .iter()
            .any(|e| e.session == killed.session_id && e.kind == EventKind::End));
    }

    #[test]
    fn the_log_keeps_the_newest_ten_thousand_and_says_it_was_truncated() {
        let dir = enabled_dir();
        let p = dir.path();
        let s = start_session(p, nonce(0)).unwrap();
        // Seeded directly: 10,000 writes of a whole file would make this test slow, not surer.
        with_state(p, |state| {
            for n in 0..MAX_EVENTS as u64 {
                state.events.push(Event {
                    schema: SCHEMA,
                    experiment: EXPERIMENT_ID.to_owned(),
                    variant: "new_mix".to_owned(),
                    session: s.session_id.clone(),
                    kind: EventKind::AlternativeRequested,
                    elapsed_ms: n,
                    changed: 0,
                    exhausted: 0,
                    overridden: false,
                    completed: false,
                });
            }
            Ok(())
        })
        .unwrap();
        record(p, &s.session_id, EventKind::Undo, 99_999, 0, 0, false).unwrap();
        let st = status(p);
        assert_eq!(st.events as usize, MAX_EVENTS);
        assert_eq!(st.dropped, 1);
        let out = dir.path().join("out.json");
        export(p, &out).unwrap();
        let exported: Export = serde_json::from_slice(&std::fs::read(out).unwrap()).unwrap();
        assert!(exported.truncated);
        assert_eq!(exported.events[0].elapsed_ms, 1, "the oldest went first");
        assert_eq!(exported.events.last().unwrap().kind, EventKind::Undo);
    }

    /// The export's keys are exactly the allowlist, and no field can carry household text.
    #[test]
    fn the_export_holds_only_allowlisted_fields() {
        let dir = enabled_dir();
        let p = dir.path();
        let s = start_session(p, nonce(0)).unwrap();
        record(
            p,
            &s.session_id,
            EventKind::AlternativeResult,
            7,
            3,
            1,
            false,
        )
        .unwrap();
        let out = dir.path().join("out.json");
        export(p, &out).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        let top: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            top,
            ["dropped", "events", "experiment", "schema", "truncated"]
        );
        let event = &value["events"][0];
        let keys: Vec<&str> = event
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            [
                "changed",
                "completed",
                "elapsed_ms",
                "exhausted",
                "experiment",
                "kind",
                "overridden",
                "schema",
                "session",
                "variant"
            ]
        );
        // Every string value is one of a closed vocabulary or the minted hex session id.
        let text = std::fs::read_to_string(&out).unwrap();
        for forbidden in ["household", "recipe", "2026-", "dinner", "title"] {
            assert!(
                !text.contains(forbidden),
                "{forbidden} leaked into the export"
            );
        }
        assert!(s.session_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// A tampered file with a smuggled field is not read as state: it starts over.
    #[test]
    fn a_file_with_unknown_fields_is_not_trusted() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(FILE_NAME),
            r#"{"schema":1,"enabled":true,"next_enabled":null,"assigned":"remix",
               "override_label":null,"next_override":null,"session":null,"events":[],
               "dropped":0,"household":"h-1"}"#,
        )
        .unwrap();
        let st = status(dir.path());
        assert!(!st.enabled);
        assert_eq!(st.assigned, None);
    }

    #[test]
    fn reset_clears_assignment_and_events_and_an_unknown_label_is_refused() {
        let dir = enabled_dir();
        let p = dir.path();
        let s = start_session(p, nonce(1)).unwrap();
        record(p, &s.session_id, EventKind::Exposure, 1, 0, 0, false).unwrap();
        reset(p).unwrap();
        let st = status(p);
        assert_eq!((st.events, st.assigned, st.enabled), (0, None, true));
        assert!(matches!(
            set_override(p, Some("Buy now".to_owned())),
            Err(ExperimentError::UnknownLabel(_))
        ));
    }
}
