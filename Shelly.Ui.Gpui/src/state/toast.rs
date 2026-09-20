use gpui::*;
use std::time::{Duration, Instant};

pub const TOAST_DISPLAY_DURATION: Duration = Duration::from_millis(3500);
pub const TOAST_MAX_VISIBLE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    pub fn icon(&self) -> &'static str {
        match self {
            ToastKind::Info => "ℹ️",
            ToastKind::Success => "✓",
            ToastKind::Warning => "⚠️",
            ToastKind::Error => "✕",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastAction {
    OpenLogs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLifecycle {
    Entering,
    Visible,
    Exiting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub title: String,
    pub message: String,
    pub action: Option<ToastAction>,
    pub lifecycle: ToastLifecycle,
    pub created_at: Instant,
}

pub struct ToastCenter {
    pub next_id: u64,
    pub toasts: Vec<Toast>,
}

impl ToastCenter {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            toasts: Vec::new(),
        }
    }

    /// Ajout pur d'un toast avec éviction déterministe préservant les erreurs
    pub fn push_toast(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
        action: Option<ToastAction>,
        now: Instant,
        reduce_motion: bool,
    ) -> u64 {
        self.next_id += 1;
        let id = self.next_id;

        if self.toasts.len() >= TOAST_MAX_VISIBLE {
            // Éviction prioritaire du plus ancien toast qui n'est PAS une erreur
            let evict_idx = self
                .toasts
                .iter()
                .position(|t| t.kind != ToastKind::Error)
                .unwrap_or(0);
            self.toasts.remove(evict_idx);
        }

        let lifecycle = if reduce_motion {
            ToastLifecycle::Visible
        } else {
            ToastLifecycle::Entering
        };

        self.toasts.push(Toast {
            id,
            kind,
            title: title.into(),
            message: message.into(),
            action,
            lifecycle,
            created_at: now,
        });

        id
    }

    pub fn set_lifecycle(&mut self, id: u64, lifecycle: ToastLifecycle) {
        if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
            toast.lifecycle = lifecycle;
        }
    }

    pub fn remove_toast(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Poste un toast dans l'entité et planifie son cycle de vie via l'exécuteur GPUI
    pub fn post(
        &mut self,
        kind: ToastKind,
        title: impl Into<String>,
        message: impl Into<String>,
        action: Option<ToastAction>,
        reduce_motion: bool,
        cx: &mut Context<Self>,
    ) -> u64 {
        let id = self.push_toast(kind, title, message, action, Instant::now(), reduce_motion);
        cx.notify();

        if reduce_motion {
            // Sous reduced_motion : pas d'animation d'entrée ou de sortie, disparition directe après 3.5s
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(TOAST_DISPLAY_DURATION).await;
                let _ = this.update(cx, |center, cx| {
                    center.remove_toast(id);
                    cx.notify();
                });
            })
            .detach();
        } else {
            // Transition d'entrée -> Visible (120ms)
            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(120))
                    .await;
                let _ = this.update(cx, |center, cx| {
                    center.set_lifecycle(id, ToastLifecycle::Visible);
                    cx.notify();
                });

                // Durée d'affichage (3.5s)
                cx.background_executor().timer(TOAST_DISPLAY_DURATION).await;
                let _ = this.update(cx, |center, cx| {
                    center.set_lifecycle(id, ToastLifecycle::Exiting);
                    cx.notify();
                });

                // Transition de sortie (160ms) puis suppression
                cx.background_executor()
                    .timer(Duration::from_millis(160))
                    .await;
                let _ = this.update(cx, |center, cx| {
                    center.remove_toast(id);
                    cx.notify();
                });
            })
            .detach();
        }

        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_toast_monotonic_ids() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id1 = center.push_toast(ToastKind::Info, "T1", "M1", None, now, false);
        let id2 = center.push_toast(ToastKind::Success, "T2", "M2", None, now, false);
        let id3 = center.push_toast(ToastKind::Error, "T3", "M3", None, now, false);

        assert!(id2 > id1);
        assert!(id3 > id2);
        assert_eq!(center.toasts.len(), 3);
    }

    #[test]
    fn test_toast_max_visible_bound() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        center.push_toast(ToastKind::Info, "T1", "M1", None, now, false);
        center.push_toast(ToastKind::Info, "T2", "M2", None, now, false);
        center.push_toast(ToastKind::Info, "T3", "M3", None, now, false);
        assert_eq!(center.toasts.len(), TOAST_MAX_VISIBLE);

        // Fourth toast must evict oldest
        let id4 = center.push_toast(ToastKind::Info, "T4", "M4", None, now, false);
        assert_eq!(center.toasts.len(), TOAST_MAX_VISIBLE);
        assert_eq!(center.toasts.last().unwrap().id, id4);
        assert!(!center.toasts.iter().any(|t| t.title == "T1"));
    }

    #[test]
    fn test_toast_deterministic_eviction_preserves_error() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        // Insert Error first, then Info, then Success
        let err_id = center.push_toast(ToastKind::Error, "Fatal Error", "E1", None, now, false);
        center.push_toast(ToastKind::Info, "Notice", "I1", None, now, false);
        center.push_toast(ToastKind::Success, "Done", "S1", None, now, false);
        assert_eq!(center.toasts.len(), 3);

        // Fourth insertion should evict the oldest NON-error toast (which is "Notice"), preserving "Fatal Error"
        center.push_toast(ToastKind::Info, "New Notice", "I2", None, now, false);
        assert_eq!(center.toasts.len(), 3);
        assert!(
            center.toasts.iter().any(|t| t.id == err_id),
            "Deterministic eviction must preserve error toast when non-errors are available"
        );
        assert!(
            !center.toasts.iter().any(|t| t.title == "Notice"),
            "Oldest non-error toast should have been evicted"
        );
    }

    #[test]
    fn test_toast_lifecycle_transitions() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id = center.push_toast(ToastKind::Info, "T", "M", None, now, false);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Entering);

        center.set_lifecycle(id, ToastLifecycle::Visible);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Visible);

        center.set_lifecycle(id, ToastLifecycle::Exiting);
        assert_eq!(center.toasts[0].lifecycle, ToastLifecycle::Exiting);

        center.remove_toast(id);
        assert!(center.toasts.is_empty());
    }

    #[test]
    fn test_toast_timer_target_identity() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id1 = center.push_toast(ToastKind::Info, "T1", "M1", None, now, false);
        let id2 = center.push_toast(ToastKind::Info, "T2", "M2", None, now, false);

        // Dismissing id1 does not affect id2
        center.remove_toast(id1);
        assert_eq!(center.toasts.len(), 1);
        assert_eq!(center.toasts[0].id, id2);
    }

    #[test]
    fn test_toast_reduced_motion_immediate() {
        let mut center = ToastCenter::new();
        let now = Instant::now();
        let id = center.push_toast(ToastKind::Success, "T", "M", None, now, true);
        assert_eq!(
            center.toasts[0].lifecycle,
            ToastLifecycle::Visible,
            "Under reduced motion, toast lifecycle must immediately snap to Visible"
        );
        assert_eq!(center.toasts[0].id, id);
    }
}
