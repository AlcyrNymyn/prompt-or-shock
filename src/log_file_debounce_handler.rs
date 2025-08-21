use std::sync::Arc;

use notify_debouncer_mini::DebounceEventHandler;
use tokio::time::Instant;

use crate::{app::SyncedSettings, log_file_reader::LogFileReader, pishock_client::PiShockClient};

pub struct LogFileDebounceHandler {
    pishock_clinet: PiShockClient,
    reader: LogFileReader,
    expected_string: String,
    settings: Arc<SyncedSettings>,
    tokio_runtime: tokio::runtime::Runtime,
}

impl LogFileDebounceHandler {
    pub fn new(
        pishock_clinet: PiShockClient,
        reader: LogFileReader,
        expected_string: String,
        settings: Arc<SyncedSettings>,
    ) -> Self {
        Self {
            pishock_clinet,
            reader,
            expected_string,
            settings,
            tokio_runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }

    fn shock_with_delay(&self, intensity: u8, death_shock: bool) {
        let client_copy = self.pishock_clinet.clone();
        let settings = self.settings.clone();
        self.tokio_runtime.spawn(async move {
            if death_shock {
                let delay = settings.death_shock_delay();
                if delay > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
                }
            }
            if settings.enable_warning_vibrate() {
                println!("Warning Vibrate...");
                let before_time = Instant::now();
                if let Err(e) = client_copy.vibrate(25, 1).await {
                    println!("Vibrate command error: {e:?}");
                    return;
                }

                // This duration is used as an estimate for how long the Shock request will take.
                let duration = Instant::now().duration_since(before_time);
                // If the estimated duration is less than 1 second, sleep for the difference.
                if duration.as_secs() < 1 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1) - duration).await;
                }
            }
            println!("Shocking...");
            if let Err(e) = client_copy.shock(intensity, 1).await {
                println!("Shock command error: {e:?}");
            }
        });
    }
}

impl DebounceEventHandler for LogFileDebounceHandler {
    fn handle_event(&mut self, _: notify_debouncer_mini::DebounceEventResult) {
        while let Some(line) = self.reader.read_line() {
            if line.ends_with(&self.expected_string) {
                println!("Life Loss Detected");
                self.shock_with_delay(self.settings.life_loss_shock(), false);
            } else if line.ends_with("Local Death") {
                println!("Death Detected");
                self.shock_with_delay(self.settings.death_shock(), true);
            }
        }
    }
}
