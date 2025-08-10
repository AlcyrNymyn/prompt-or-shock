use std::io::{Read, Write};

use crate::log_file_debounce_handler::LogFileDebounceHandler;
use crate::log_file_reader::LogFileReader;
use crate::pishock_client::PiShockClient;

use eframe::egui;
use notify_debouncer_mini::{Debouncer, new_debouncer, notify::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
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
pub struct PromptOrShockApp {
    settings: Settings,
    debouncer: Option<Debouncer<ReadDirectoryChangesWatcher>>,
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
        Self {
            settings,
            debouncer: None,
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
            self.settings.life_loss_shock,
            self.settings.death_shock,
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let vrc_username_label = ui.label("VRChat Username: ");
                ui.text_edit_singleline(&mut self.settings.vrc_username)
                    .labelled_by(vrc_username_label.id);
            });
            ui.horizontal(|ui| {
                let pishock_username_label = ui.label("PiShock Username: ");
                ui.text_edit_singleline(&mut self.settings.pishock_username)
                    .labelled_by(pishock_username_label.id);
            });
            ui.horizontal(|ui| {
                let pishock_api_key_label = ui.label("PiShock API Key: ");
                ui.text_edit_singleline(&mut self.settings.pishock_api_key)
                    .labelled_by(pishock_api_key_label.id);
            });
            ui.horizontal(|ui| {
                let shocker_share_code_label = ui.label("Shocker Share Code: ");
                ui.text_edit_singleline(&mut self.settings.shocker_share_code)
                    .labelled_by(shocker_share_code_label.id);
            });
            ui.add(
                egui::Slider::new(&mut self.settings.life_loss_shock, 0..=100)
                    .text("Life Loss Shock"),
            );
            ui.add(egui::Slider::new(&mut self.settings.death_shock, 0..=100).text("Death Shock"));

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
        });
    }
}
