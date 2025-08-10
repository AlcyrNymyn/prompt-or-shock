use notify_debouncer_mini::DebounceEventHandler;

use crate::{log_file_reader::LogFileReader, pishock_client::PiShockClient};

pub struct LogFileDebounceHandler {
    pishock_clinet: PiShockClient,
    reader: LogFileReader,
    expected_string: String,
    life_loss_shock: u8,
    death_shock: u8,
    tokio_runtime: tokio::runtime::Runtime,
}

impl LogFileDebounceHandler {
    pub fn new(
        pishock_clinet: PiShockClient,
        reader: LogFileReader,
        expected_string: String,
        life_loss_shock: u8,
        death_shock: u8,
    ) -> Self {
        Self {
            pishock_clinet,
            reader,
            expected_string,
            life_loss_shock,
            death_shock,
            tokio_runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }

    fn shock_with_delay(&self, intensity: u8) {
        let client_copy = self.pishock_clinet.clone();
        self.tokio_runtime.spawn(async move {
            println!("Warning Vibrate...");
            if let Err(e) = client_copy.vibrate(25, 1).await {
                println!("Vibrate command error: {e:?}");
                return;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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
                self.shock_with_delay(self.life_loss_shock);
            } else if line.ends_with("Local Death") {
                println!("Death Detected");
                self.shock_with_delay(self.death_shock);
            }
        }
    }
}
