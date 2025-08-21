use std::{
    io::{Read, Write},
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

use crate::log_file_debounce_handler::LogFileDebounceHandler;
use crate::log_file_reader::LogFileReader;
use crate::pishock_client::PiShockClient;

use eframe::egui;
use notify_debouncer_mini::{Debouncer, new_debouncer, notify::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Settings {
    vrc_username: String,
    pishock_username: String,
    pishock_api_key: String,
    shocker_share_code: String,
    #[serde(default)]
    life_loss_shock: u8,
    #[serde(default)]
    death_shock: u8,
}

pub struct SyncedSettings {
    life_loss_shock: AtomicU8,
    death_shock: AtomicU8,
}

impl SyncedSettings {
    fn from_settings(settings: &Settings) -> Arc<Self> {
        Arc::new(Self {
            life_loss_shock: AtomicU8::new(settings.life_loss_shock),
            death_shock: AtomicU8::new(settings.death_shock),
        })
    }

    pub fn life_loss_shock(&self) -> u8 {
        self.life_loss_shock.load(Ordering::Relaxed)
    }

    pub fn death_shock(&self) -> u8 {
        self.death_shock.load(Ordering::Relaxed)
    }
}

pub struct PromptOrShockApp {
    settings: Settings,
    debouncer: Option<Debouncer<ReadDirectoryChangesWatcher>>,
    synced_settings: Arc<SyncedSettings>,
    last_update: Option<Instant>,
}

impl Default for PromptOrShockApp {
    fn default() -> Self {
        let settings = if let Ok(true) = std::fs::exists("./settings.json") {
            let mut file = std::fs::File::open("./settings.json").unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            serde_json::from_str(&contents).unwrap()
        } else {
            Settings {
                vrc_username: String::default(),
                pishock_username: String::default(),
                pishock_api_key: String::default(),
                shocker_share_code: String::default(),
                life_loss_shock: 0,
                death_shock: 0,
            }
        };
        let synced_settings = SyncedSettings::from_settings(&settings);
        Self {
            settings,
            debouncer: None,
            synced_settings,
            last_update: None,
        }
    }
}

impl PromptOrShockApp {
    fn save_settings(&self) {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./settings.json")
            .unwrap();
        let settings_json = serde_json::to_string(&self.settings).unwrap();
        file.write_all(settings_json.as_bytes()).unwrap();
    }

    fn begin_watch(&mut self) {
        let pishock_client = PiShockClient::new(
            self.settings.pishock_username.clone(),
            self.settings.pishock_api_key.clone(),
            self.settings.shocker_share_code.clone(),
        );
        let expected_str = format!("Shocked: {}", self.settings.vrc_username);
        let file_reader = LogFileReader::new();
        let file_path = file_reader.log_file_path().clone();
        let debug_handler = LogFileDebounceHandler::new(
            pishock_client,
            file_reader,
            expected_str,
            self.synced_settings.clone(),
        );
        let mut debouncer =
            new_debouncer(std::time::Duration::from_millis(100), debug_handler).unwrap();
        debouncer
            .watcher()
            .watch(&file_path, RecursiveMode::NonRecursive)
            .expect("Unable to watch log file.");
        self.debouncer = Some(debouncer);
    }
}

impl eframe::App for PromptOrShockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut is_dirty = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let vrc_username_label = ui.label("VRChat Username: ");
                is_dirty |= ui
                    .text_edit_singleline(&mut self.settings.vrc_username)
                    .labelled_by(vrc_username_label.id)
                    .changed();
            });
            ui.horizontal(|ui| {
                let pishock_username_label = ui.label("PiShock Username: ");
                is_dirty |= ui
                    .text_edit_singleline(&mut self.settings.pishock_username)
                    .labelled_by(pishock_username_label.id)
                    .changed();
            });
            ui.horizontal(|ui| {
                let pishock_api_key_label = ui.label("PiShock API Key: ");
                is_dirty |= ui
                    .text_edit_singleline(&mut self.settings.pishock_api_key)
                    .labelled_by(pishock_api_key_label.id)
                    .changed();
            });
            ui.horizontal(|ui| {
                let shocker_share_code_label = ui.label("Shocker Share Code: ");
                is_dirty |= ui
                    .text_edit_singleline(&mut self.settings.shocker_share_code)
                    .labelled_by(shocker_share_code_label.id)
                    .changed();
            });

            if self.debouncer.is_some() {
                if ui.button("Stop").clicked() {
                    drop(self.debouncer.take());
                }
            } else {
                if ui.button("Start").clicked() {
                    self.save_settings();
                    self.begin_watch();
                }
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.settings.life_loss_shock, 0..=100)
                        .text("Life Loss Shock"),
                )
                .changed()
            {
                is_dirty = true;
                self.synced_settings
                    .life_loss_shock
                    .store(self.settings.life_loss_shock, Ordering::Relaxed);
            }
            if ui
                .add(egui::Slider::new(&mut self.settings.death_shock, 0..=100).text("Death Shock"))
                .changed()
            {
                is_dirty = true;
                self.synced_settings
                    .death_shock
                    .store(self.settings.death_shock, Ordering::Relaxed);
            }
        });

        // Debounce changes - will save 3 seconds after last change.
        if is_dirty {
            self.last_update = Some(Instant::now());
        } else if self
            .last_update
            .is_some_and(|t| Instant::now().duration_since(t) > Duration::new(3, 0))
        {
            self.last_update = None;
            self.save_settings();
        }
    }
}
