//! Coordenação de streams IPC via `tauri::ipc::Channel` (Spec 033, CA-9).
//! Emite frames em ~30 Hz quando visível e suspende entregas a 0 Hz quando minimizado.

use crate::commands::invoke_dispatch;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sonix_desktop::api::ApiContext;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryFrame {
    pub seq: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunerFrame {
    pub seq: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeFrame {
    pub seq: u64,
    pub mode: String,
    pub payload: Value,
}

struct ActiveChannels {
    telemetry: Channel<TelemetryFrame>,
    tuner: Channel<TunerFrame>,
    scope: Channel<ScopeFrame>,
}

/// Coordenador de visibilidade e sequência monotônica para streams Tauri.
pub struct StreamCoordinator {
    visible: AtomicBool,
    seq_telemetry: AtomicU64,
    seq_tuner: AtomicU64,
    seq_scope: AtomicU64,
    scope_mode: Mutex<String>,
    channels: Arc<Mutex<Option<ActiveChannels>>>,
    loop_started: AtomicBool,
}

impl Default for StreamCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamCoordinator {
    pub fn new() -> Self {
        Self {
            visible: AtomicBool::new(true),
            seq_telemetry: AtomicU64::new(0),
            seq_tuner: AtomicU64::new(0),
            seq_scope: AtomicU64::new(0),
            scope_mode: Mutex::new("transfer".to_string()),
            channels: Arc::new(Mutex::new(None)),
            loop_started: AtomicBool::new(false),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub fn set_visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::Relaxed);
    }

    pub fn set_scope_mode(&self, mode: &str) {
        if let Ok(mut guard) = self.scope_mode.lock() {
            *guard = mode.to_string();
        }
    }

    pub fn next_telemetry_frame(&self, ctx: &ApiContext) -> Option<TelemetryFrame> {
        if !self.is_visible() {
            return None;
        }
        let resp = invoke_dispatch(ctx, "GET", "/api/telemetry", b"");
        let payload: Value = serde_json::from_str(&resp.body).ok()?;
        let seq = self.seq_telemetry.fetch_add(1, Ordering::Relaxed) + 1;
        Some(TelemetryFrame { seq, payload })
    }

    pub fn next_tuner_frame(&self, ctx: &ApiContext) -> Option<TunerFrame> {
        if !self.is_visible() {
            return None;
        }
        let resp = invoke_dispatch(ctx, "GET", "/api/tuner", b"");
        let payload: Value = serde_json::from_str(&resp.body).ok()?;
        let seq = self.seq_tuner.fetch_add(1, Ordering::Relaxed) + 1;
        Some(TunerFrame { seq, payload })
    }

    pub fn next_scope_frame(&self, ctx: &ApiContext, mode: &str) -> Option<ScopeFrame> {
        if !self.is_visible() {
            return None;
        }
        let route = format!("/api/scope?mode={}", mode);
        let resp = invoke_dispatch(ctx, "GET", &route, b"");
        let payload: Value = serde_json::from_str(&resp.body).ok()?;
        let seq = self.seq_scope.fetch_add(1, Ordering::Relaxed) + 1;
        Some(ScopeFrame {
            seq,
            mode: mode.to_string(),
            payload,
        })
    }

    pub fn start_channels(
        self: &Arc<Self>,
        ctx: ApiContext,
        telemetry: Channel<TelemetryFrame>,
        tuner: Channel<TunerFrame>,
        scope: Channel<ScopeFrame>,
    ) {
        if let Ok(mut guard) = self.channels.lock() {
            *guard = Some(ActiveChannels {
                telemetry,
                tuner,
                scope,
            });
        }
        if self
            .loop_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let coordinator = Arc::clone(self);
        std::thread::spawn(move || {
            let mut tuner_divider = 0u8;
            loop {
                std::thread::sleep(Duration::from_millis(33));
                if !coordinator.is_visible() {
                    continue;
                }
                let channels_guard = match coordinator.channels.lock() {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                let Some(ref ch) = *channels_guard else {
                    continue;
                };

                if let Some(frame) = coordinator.next_telemetry_frame(&ctx) {
                    let _ = ch.telemetry.send(frame);
                }

                tuner_divider = tuner_divider.wrapping_add(1);
                if tuner_divider % 2 == 0 {
                    if let Some(t_frame) = coordinator.next_tuner_frame(&ctx) {
                        let _ = ch.tuner.send(t_frame);
                    }
                    let mode = coordinator
                        .scope_mode
                        .lock()
                        .map(|g| g.clone())
                        .unwrap_or_else(|_| "transfer".to_string());
                    if let Some(s_frame) = coordinator.next_scope_frame(&ctx, &mode) {
                        let _ = ch.scope.send(s_frame);
                    }
                }
            }
        });
    }
}
