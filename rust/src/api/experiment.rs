//! The tester-only wording experiment (OPT-007 §10) across the bridge. `dir` is the app-support
//! directory Dart resolves; the state file there is separate from the household database. No
//! call here reads or writes household data, and none is an input to planning.

use std::path::PathBuf;

use crate::api::error::KimattaError;
use crate::experiment::{self, EventKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentLabelDto {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentSessionDto {
    pub session_id: String,
    pub label_id: String,
    pub label: String,
    pub active: bool,
    pub overridden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentEventKindDto {
    Exposure,
    AlternativeRequested,
    AlternativeResult,
    Undo,
    AcceptSuccess,
    Discard,
    Background,
    Resume,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentEventDto {
    pub session_id: String,
    pub kind: ExperimentEventKindDto,
    pub elapsed_ms: u64,
    pub changed: u32,
    pub exhausted: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentStatusDto {
    pub enabled: bool,
    /// `Some` when a change waits for the next session.
    pub next_enabled: Option<bool>,
    pub assigned: Option<String>,
    pub override_label: Option<String>,
    /// Outer `Some`: a pending override change; inner `None` clears it.
    pub override_pending: bool,
    pub next_override: Option<String>,
    pub events: u32,
    pub dropped: u64,
    pub labels: Vec<ExperimentLabelDto>,
}

fn io(e: experiment::ExperimentError) -> KimattaError {
    KimattaError::Storage {
        message: e.to_string(),
    }
}

fn nonce() -> [u8; 16] {
    *uuid::Uuid::new_v4().as_bytes()
}

pub fn experiment_start_session(dir: String) -> Result<ExperimentSessionDto, KimattaError> {
    let s = experiment::start_session(&PathBuf::from(dir), nonce()).map_err(io)?;
    Ok(ExperimentSessionDto {
        session_id: s.session_id,
        label_id: s.label_id,
        label: s.label,
        active: s.active,
        overridden: s.overridden,
    })
}

/// Returns whether the event was kept (see `experiment::record`).
pub fn experiment_record(dir: String, event: ExperimentEventDto) -> Result<bool, KimattaError> {
    let kind = match event.kind {
        ExperimentEventKindDto::Exposure => EventKind::Exposure,
        ExperimentEventKindDto::AlternativeRequested => EventKind::AlternativeRequested,
        ExperimentEventKindDto::AlternativeResult => EventKind::AlternativeResult,
        ExperimentEventKindDto::Undo => EventKind::Undo,
        ExperimentEventKindDto::AcceptSuccess => EventKind::AcceptSuccess,
        ExperimentEventKindDto::Discard => EventKind::Discard,
        ExperimentEventKindDto::Background => EventKind::Background,
        ExperimentEventKindDto::Resume => EventKind::Resume,
        ExperimentEventKindDto::End => EventKind::End,
    };
    experiment::record(
        &PathBuf::from(dir),
        &event.session_id,
        kind,
        event.elapsed_ms,
        event.changed,
        event.exhausted,
        event.completed,
    )
    .map_err(io)
}

pub fn experiment_status(dir: String) -> ExperimentStatusDto {
    let s = experiment::status(&PathBuf::from(dir));
    ExperimentStatusDto {
        enabled: s.enabled,
        next_enabled: s.next_enabled,
        assigned: s.assigned,
        override_label: s.override_label,
        override_pending: s.next_override.is_some(),
        next_override: s.next_override.flatten(),
        events: s.events,
        dropped: s.dropped,
        labels: experiment::LABELS
            .iter()
            .map(|(id, label)| ExperimentLabelDto {
                id: (*id).to_owned(),
                label: (*label).to_owned(),
            })
            .collect(),
    }
}

pub fn experiment_set_enabled(dir: String, enabled: bool) -> Result<(), KimattaError> {
    experiment::set_enabled(&PathBuf::from(dir), enabled).map_err(io)
}

pub fn experiment_set_override(dir: String, label_id: Option<String>) -> Result<(), KimattaError> {
    experiment::set_override(&PathBuf::from(dir), label_id).map_err(io)
}

pub fn experiment_reset(dir: String) -> Result<(), KimattaError> {
    experiment::reset(&PathBuf::from(dir)).map_err(io)
}

/// Writes the sanitized record — never the database — to `dest_path`; returns the event count.
pub fn experiment_export(dir: String, dest_path: String) -> Result<u32, KimattaError> {
    experiment::export(&PathBuf::from(dir), &PathBuf::from(dest_path)).map_err(io)
}
