//! The capture pipeline.
//!
//! Platform backends call [`CapturePipeline::handle_change`] after the OS
//! reports a change. Everything after that is platform independent and tested
//! with fakes.

use std::sync::{Arc, RwLock};

use copyfie_core::hash;
use copyfie_core::model::NewItem;
use copyfie_platform::{ClipboardReader, ForegroundApp};

use crate::capture::events::HistoryEvents;
use crate::capture::privacy::{CaptureConfig, GateDecision, Platform, PrivacyGate, SkipReason};
use crate::capture::suppress::Suppressor;
use crate::capture::unix_ms;
use crate::store::StoreHandle;

use super::CaptureError;

/// Injectable wall clock (Unix ms).
pub type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

/// What happened to one clipboard change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureOutcome {
    Stored { id: i64, is_new: bool },
    Skipped(SkipReason),
}

/// Capture pipeline for one platform backend.
pub struct CapturePipeline<R, F, E> {
    reader: R,
    foreground: F,
    store: StoreHandle,
    config: Arc<RwLock<CaptureConfig>>,
    suppressor: Arc<Suppressor>,
    events: E,
    platform: Platform,
    clock: Clock,
}

impl<R, F, E> CapturePipeline<R, F, E>
where
    R: ClipboardReader,
    F: ForegroundApp,
    E: HistoryEvents,
{
    pub fn new(
        reader: R,
        foreground: F,
        store: StoreHandle,
        config: Arc<RwLock<CaptureConfig>>,
        suppressor: Arc<Suppressor>,
        events: E,
    ) -> Self {
        Self {
            reader,
            foreground,
            store,
            config,
            suppressor,
            events,
            platform: Platform::current(),
            clock: Arc::new(unix_ms),
        }
    }

    /// Override the detected platform (tests, diagnostics).
    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// Override the wall clock (tests).
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// Shared config handle for settings commands.
    pub fn config(&self) -> Arc<RwLock<CaptureConfig>> {
        Arc::clone(&self.config)
    }

    /// Handle one reported clipboard change.
    ///
    /// Errors are returned to the caller (the platform loop logs and continues);
    /// skipped changes are normal outcomes, not errors.
    pub fn handle_change(&self, sequence: u64) -> Result<CaptureOutcome, CaptureError> {
        // Phase 1: formats only. No payload is read yet.
        let probe = self.reader.probe(sequence)?;
        let config = self.config_snapshot();
        let source = self.foreground.current();

        match PrivacyGate::evaluate(&config, &probe, source.as_ref(), self.platform) {
            GateDecision::Allow => {}
            GateDecision::Skip(reason) => return Ok(CaptureOutcome::Skipped(reason)),
        }

        // Phase 2: the gate allowed this change, so read the payload.
        let Some(raw_text) = self.reader.read_text(sequence)? else {
            return Ok(CaptureOutcome::Skipped(SkipReason::NoText));
        };

        let text = hash::canonicalize_text(&raw_text);
        if text.trim().is_empty() {
            return Ok(CaptureOutcome::Skipped(SkipReason::EmptyText));
        }

        let content_hash = hash::hash_bytes(text.as_bytes());
        if self
            .suppressor
            .consume_if_suppressed(&content_hash, std::time::Instant::now())
        {
            return Ok(CaptureOutcome::Skipped(SkipReason::Echo));
        }

        let item = NewItem::text(
            &text,
            source.unwrap_or_default(),
            (self.clock)(),
            config.retention,
        );

        let outcome = self.store.call_blocking(move |db| db.upsert(&item))?;

        // Only announce after the write succeeded: persistence acknowledges the
        // capture, which is what makes dedupe safe.
        self.events.changed();

        Ok(CaptureOutcome::Stored {
            id: outcome.id,
            is_new: outcome.is_new,
        })
    }

    fn config_snapshot(&self) -> CaptureConfig {
        self.config
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use copyfie_core::retention::Retention;
    use copyfie_core::Database;
    use copyfie_core::SourceApp;
    use copyfie_platform::{ClipboardProbe, PlatformError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeReader {
        probe: ClipboardProbe,
        text: Option<String>,
        probes: AtomicUsize,
        reads: AtomicUsize,
        fail_probe: bool,
    }

    impl FakeReader {
        fn new(probe: ClipboardProbe, text: Option<&str>) -> Self {
            Self {
                probe,
                text: text.map(str::to_string),
                ..Self::default()
            }
        }

        fn probe_count(&self) -> usize {
            self.probes.load(Ordering::SeqCst)
        }

        fn read_count(&self) -> usize {
            self.reads.load(Ordering::SeqCst)
        }
    }

    impl ClipboardReader for FakeReader {
        fn probe(&self, _sequence: u64) -> Result<ClipboardProbe, PlatformError> {
            self.probes.fetch_add(1, Ordering::SeqCst);
            if self.fail_probe {
                return Err(PlatformError::Clipboard("probe failed".into()));
            }
            Ok(self.probe)
        }

        fn read_text(&self, _sequence: u64) -> Result<Option<String>, PlatformError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            Ok(self.text.clone())
        }
    }

    struct FakeForeground(Option<SourceApp>);

    impl ForegroundApp for FakeForeground {
        fn current(&self) -> Option<SourceApp> {
            self.0.clone()
        }
    }

    #[derive(Default)]
    struct RecordingEvents {
        count: AtomicUsize,
        seen: Mutex<Vec<()>>,
    }

    impl RecordingEvents {
        fn count(&self) -> usize {
            self.count.load(Ordering::SeqCst)
        }
    }

    impl HistoryEvents for RecordingEvents {
        fn changed(&self) {
            self.count.fetch_add(1, Ordering::SeqCst);
            self.seen.lock().expect("lock").push(());
        }
    }

    struct Harness {
        pipeline: CapturePipeline<Arc<FakeReader>, FakeForeground, Arc<RecordingEvents>>,
        reader: Arc<FakeReader>,
        events: Arc<RecordingEvents>,
        store: StoreHandle,
        suppressor: Arc<Suppressor>,
        config: Arc<RwLock<CaptureConfig>>,
    }

    fn harness(probe: ClipboardProbe, text: Option<&str>) -> Harness {
        harness_with(probe, text, None, CaptureConfig::default())
    }

    fn harness_with(
        probe: ClipboardProbe,
        text: Option<&str>,
        source: Option<SourceApp>,
        config: CaptureConfig,
    ) -> Harness {
        let store = StoreHandle::spawn(Database::open_in_memory().expect("db"));
        let reader = Arc::new(FakeReader::new(probe, text));
        let events = Arc::new(RecordingEvents::default());
        let suppressor = Arc::new(Suppressor::new());
        let config = Arc::new(RwLock::new(config));

        let pipeline = CapturePipeline::new(
            Arc::clone(&reader),
            FakeForeground(source),
            store.clone(),
            Arc::clone(&config),
            Arc::clone(&suppressor),
            Arc::clone(&events),
        )
        .with_clock(Arc::new(|| 1_000_000))
        .with_platform(Platform::Linux);

        Harness {
            pipeline,
            reader,
            events,
            store,
            suppressor,
            config,
        }
    }

    fn count(store: &StoreHandle) -> i64 {
        store.call_blocking(|db| db.count(None)).expect("count")
    }

    fn notepad() -> SourceApp {
        SourceApp {
            id: Some(r"C:\Apps\Notepad.exe".to_string()),
            name: Some("Notepad".to_string()),
            window_title: Some("Untitled".to_string()),
        }
    }

    #[test]
    fn stores_allowed_text_and_emits_once() {
        let harness = harness(ClipboardProbe::text(), Some("hello\r\nworld"));

        let outcome = harness.pipeline.handle_change(1).expect("capture");
        assert_eq!(
            outcome,
            CaptureOutcome::Stored {
                id: 1,
                is_new: true
            }
        );
        assert_eq!(count(&harness.store), 1);
        assert_eq!(harness.events.count(), 1);

        let row = harness
            .store
            .call_blocking(|db| db.get(1))
            .expect("get")
            .expect("row");
        assert_eq!(row.text, "hello\nworld");
        assert_eq!(row.captured_at, 1_000_000);
        assert_eq!(row.expires_at, Some(1_000_000 + 30 * 86_400_000));
    }

    #[test]
    fn concealed_change_is_probed_but_never_read() {
        let probe = ClipboardProbe {
            is_concealed: true,
            ..ClipboardProbe::text()
        };
        let harness = harness(probe, Some("hunter2"));

        let outcome = harness.pipeline.handle_change(1).expect("capture");
        assert_eq!(outcome, CaptureOutcome::Skipped(SkipReason::Concealed));
        assert_eq!(harness.reader.probe_count(), 1);
        assert_eq!(harness.reader.read_count(), 0, "payload must not be read");
        assert_eq!(count(&harness.store), 0);
        assert_eq!(harness.events.count(), 0);
    }

    #[test]
    fn paused_capture_is_not_read_or_stored() {
        let config = CaptureConfig {
            paused: true,
            ..CaptureConfig::default()
        };
        let harness = harness_with(ClipboardProbe::text(), Some("secret"), None, config);

        assert_eq!(
            harness.pipeline.handle_change(1).expect("capture"),
            CaptureOutcome::Skipped(SkipReason::Paused)
        );
        assert_eq!(harness.reader.read_count(), 0);
        assert_eq!(count(&harness.store), 0);
    }

    #[test]
    fn blocked_app_is_not_stored() {
        let config = CaptureConfig {
            blocklist: vec![crate::capture::BlocklistEntry {
                platform: Platform::Linux,
                id: "keepassxc".to_string(),
                label: None,
            }],
            ..CaptureConfig::default()
        };
        let source = SourceApp {
            id: Some("/usr/bin/keepassxc".to_string()),
            name: Some("KeePassXC".to_string()),
            window_title: None,
        };
        let harness = harness_with(
            ClipboardProbe::text(),
            Some("password"),
            Some(source),
            config,
        );

        assert_eq!(
            harness.pipeline.handle_change(1).expect("capture"),
            CaptureOutcome::Skipped(SkipReason::BlockedApp)
        );
        assert_eq!(harness.reader.read_count(), 0);
        assert_eq!(count(&harness.store), 0);
    }

    #[test]
    fn empty_and_whitespace_text_is_skipped() {
        for payload in ["", "   \n\t "] {
            let harness = harness(ClipboardProbe::text(), Some(payload));
            assert_eq!(
                harness.pipeline.handle_change(1).expect("capture"),
                CaptureOutcome::Skipped(SkipReason::EmptyText)
            );
            assert_eq!(count(&harness.store), 0);
            assert_eq!(harness.events.count(), 0);
        }

        let no_text = harness(ClipboardProbe::text(), None);
        assert_eq!(
            no_text.pipeline.handle_change(1).expect("capture"),
            CaptureOutcome::Skipped(SkipReason::NoText)
        );
    }

    #[test]
    fn armed_echo_is_suppressed_once() {
        let harness = harness(ClipboardProbe::text(), Some("pasted"));

        harness
            .suppressor
            .arm_text("pasted", std::time::Instant::now());
        assert_eq!(
            harness.pipeline.handle_change(1).expect("capture"),
            CaptureOutcome::Skipped(SkipReason::Echo)
        );
        assert_eq!(count(&harness.store), 0);

        // The token was consumed; a real re-copy is captured.
        assert!(matches!(
            harness.pipeline.handle_change(2).expect("capture"),
            CaptureOutcome::Stored { is_new: true, .. }
        ));
    }

    #[test]
    fn recopy_bumps_the_same_row() {
        let harness = harness(ClipboardProbe::text(), Some("same"));

        let first = harness.pipeline.handle_change(1).expect("first");
        let second = harness.pipeline.handle_change(2).expect("second");

        assert_eq!(
            first,
            CaptureOutcome::Stored {
                id: 1,
                is_new: true
            }
        );
        assert_eq!(
            second,
            CaptureOutcome::Stored {
                id: 1,
                is_new: false
            }
        );
        assert_eq!(count(&harness.store), 1);
        assert_eq!(harness.events.count(), 2, "both writes notify the palette");
    }

    #[test]
    fn stored_item_keeps_the_source_app() {
        let harness = harness_with(
            ClipboardProbe::text(),
            Some("from notepad"),
            Some(notepad()),
            CaptureConfig::default(),
        );

        harness.pipeline.handle_change(1).expect("capture");

        let row = harness
            .store
            .call_blocking(|db| db.get(1))
            .expect("get")
            .expect("row");
        assert_eq!(row.source_app_name.as_deref(), Some("Notepad"));
        assert_eq!(row.source_window_title.as_deref(), Some("Untitled"));
    }

    #[test]
    fn retention_setting_is_applied() {
        let config = CaptureConfig {
            retention: Retention::Days(7),
            ..CaptureConfig::default()
        };
        let harness = harness_with(ClipboardProbe::text(), Some("short"), None, config);

        harness.pipeline.handle_change(1).expect("capture");

        let row = harness
            .store
            .call_blocking(|db| db.get(1))
            .expect("get")
            .expect("row");
        assert_eq!(row.expires_at, Some(1_000_000 + 7 * 86_400_000));
    }

    #[test]
    fn config_updates_take_effect_immediately() {
        let harness = harness(ClipboardProbe::text(), Some("pausable"));
        harness.config.write().expect("lock").paused = true;

        assert_eq!(
            harness.pipeline.handle_change(1).expect("capture"),
            CaptureOutcome::Skipped(SkipReason::Paused)
        );
    }

    #[test]
    fn probe_failures_are_reported() {
        let mut reader = FakeReader::new(ClipboardProbe::text(), Some("x"));
        reader.fail_probe = true;
        let reader = Arc::new(reader);
        let store = StoreHandle::spawn(Database::open_in_memory().expect("db"));
        let events = Arc::new(RecordingEvents::default());
        let pipeline = CapturePipeline::new(
            reader,
            FakeForeground(None),
            store,
            Arc::new(RwLock::new(CaptureConfig::default())),
            Arc::new(Suppressor::new()),
            events,
        );

        let error = pipeline.handle_change(1).expect_err("probe error");
        assert!(matches!(error, CaptureError::Platform(_)));
    }
}
