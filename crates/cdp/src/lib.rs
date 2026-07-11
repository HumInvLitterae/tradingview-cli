mod client;
mod transport;

pub use client::{
    CdpClient, KeyEvent, KeyEventType, MouseEvent, MouseEventType, RuntimeEvaluator, ScreenshotClip,
};
pub use transport::{
    CdpHttpSession, Target, TargetSelection, TransportConfig, discover_target, fetch_targets,
    is_app_window_target, is_new_tab_target, is_screener_target, new_target_url, select_target,
    target_cli_args, target_title_for_handoff, target_url_for_handoff,
};

#[cfg(test)]
pub(crate) mod test_support {
    use tokio::sync::{Mutex, MutexGuard};

    pub(crate) async fn loopback_fixture_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::const_new(());
        LOCK.lock().await
    }
}
