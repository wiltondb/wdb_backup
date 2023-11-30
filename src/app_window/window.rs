/*
 * Copyright 2023, WiltonDB Software
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::path::Path;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::process::Stdio;

use chrono;

use super::*;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Default)]
pub struct AppWindow {
    pub(super) c: AppWindowControls,

    pg_conn_config: PgConnConfig,

    about_dialog_join_handle: ui::PopupJoinHandle<()>,
    connect_dialog_join_handle: ui::PopupJoinHandle<ConnectDialogResult>,
    load_join_handle: ui::PopupJoinHandle<LoadDbnamesDialogResult>,
    backup_dialog_join_handle: ui::PopupJoinHandle<BackupDialogResult>,
    restore_dialog_join_handle: ui::PopupJoinHandle<RestoreDialogResult>,
}

impl AppWindow {

    pub fn new() -> Self {
        Default::default()
    }

    pub(super) fn init(&mut self) {
        self.pg_conn_config.hostname = String::from("localhost");
        self.pg_conn_config.port = 5432;
        // todo: removeme
        self.pg_conn_config.username = String::from("wilton");
        self.pg_conn_config.password = String::from("wilton");
        self.pg_conn_config.enable_tls = true;
        self.pg_conn_config.accept_invalid_tls = true;

        self.set_status_bar_dbconn_label("none");
        self.open_connect_dialog(nwg::EventData::NoData);
    }

    pub(super) fn close(&mut self, _: nwg::EventData) {
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    pub(super) fn open_about_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = AboutDialogArgs::new(&self.c.about_notice);
        self.about_dialog_join_handle = AboutDialog::popup(args);
    }

    pub(super) fn await_about_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.about_notice.receive();
        let _ = self.about_dialog_join_handle.join();
    }

    pub(super) fn open_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = ConnectDialogArgs::new(&self.c.connect_notice, self.pg_conn_config.clone());
        self.connect_dialog_join_handle = ConnectDialog::popup(args);
    }

    pub(super) fn await_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.connect_notice.receive();
        let res = self.connect_dialog_join_handle.join();
        if !res.cancelled {
            self.set_dbnames(&res.dbnames, &res.bbf_db);
            self.pg_conn_config = res.pg_conn_config;
            let sbar_label = format!(
                "{}:{}", &self.pg_conn_config.hostname, &self.pg_conn_config.port);
            self.set_status_bar_dbconn_label(&sbar_label);
        }
    }

    pub(super) fn open_load_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let pcc = self.pg_conn_config.clone();
        let args = LoadDbnamesDialogArgs::new(&self.c.load_notice, pcc);
        self.load_join_handle = LoadDbnamesDialog::popup(args);
    }

    pub(super) fn await_load_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.load_notice.receive();
        let res = self.load_join_handle.join();
        if res.success {
            self.set_dbnames(&res.dbnames, &res.bbf_db);
        }
    }

    pub(super) fn open_backup_dialog(&mut self, _: nwg::EventData) {
        let dbname = match self.c.backup_dbname_combo.selection_string() {
            Some(name) => name,
            None => return
        };
        let dir = self.c.backup_dest_dir_input.text();
        let filename = self.c.backup_filename_input.text();
        self.c.window.set_enabled(false);
        let args = BackupDialogArgs::new(
            &self.c.backup_dialog_notice, &self.pg_conn_config,  &dbname, &dir, &filename);
        self.backup_dialog_join_handle = BackupDialog::popup(args);
    }

    pub(super) fn await_backup_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.backup_dialog_notice.receive();
        let _ = self.backup_dialog_join_handle.join();
    }

    pub(super) fn open_restore_command_dialog(&mut self, _: nwg::EventData) {
        let pcc = &self.pg_conn_config;
        let zipfile = self.c.restore_src_file_input.text();
        let dbname = self.c.restore_dbname_input.text();
        let bbf_db = self.c.restore_bbf_db_input.text();
        self.c.window.set_enabled(false);
        let args = RestoreDialogArgs::new(
            &self.c.restore_dialog_notice, &pcc,
            &zipfile, &dbname, &bbf_db);
        self.restore_dialog_join_handle = RestoreDialog::popup(args);
    }

    pub(super) fn await_restore_command_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.restore_dialog_notice.receive();
        let _ = self.restore_dialog_join_handle.join();
    }

    pub(super) fn open_website(&mut self, _: nwg::EventData) {
        let _ = Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("https://wiltondb.com")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    pub(super) fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }

    pub(super) fn choose_dest_dir(&mut self, _: nwg::EventData) {
        if let Ok(d) = std::env::current_dir() {
            if let Some(d) = d.to_str() {
                let _ = self.c.backup_dest_dir_chooser.set_default_folder(d);
            }
        }

        if self.c.backup_dest_dir_chooser.run(Some(&self.c.window)) {
            self.c.backup_dest_dir_input.set_text("");
            if let Ok(directory) = self.c.backup_dest_dir_chooser.get_selected_item() {
                let dir = directory.to_string_lossy().to_string();
                self.c.backup_dest_dir_input.set_text(&dir);
            }
        }
    }

    pub(super) fn choose_src_file(&mut self, _: nwg::EventData) {
        if let Ok(d) = std::env::current_dir() {
            if let Some(d) = d.to_str() {
                let _ = self.c.restore_src_file_chooser.set_default_folder(d);
            }
        }

        if self.c.restore_src_file_chooser.run(Some(&self.c.window)) {
            self.c.restore_src_file_input.set_text("");
            if let Ok(file) = self.c.restore_src_file_chooser.get_selected_item() {
                let fpath_st = file.to_string_lossy().to_string();
                self.c.restore_src_file_input.set_text(&fpath_st);
                if let Some(filename) = Path::new(&file).file_name() {
                    let name_st = filename.to_string_lossy().to_string();
                    let ext = match Path::new(&file).extension() {
                        Some(ext) => format!(".{}", ext.to_string_lossy().to_string()),
                        None => "".to_string()
                    };
                    let dbname: String = name_st.chars().take(name_st.len() - ext.len()).collect();
                    self.c.restore_dbname_input.set_text(&dbname);
                }
            }
        }
    }

    pub(super) fn on_dbname_changed(&mut self, _: nwg::EventData) {
        if let Some(name) = &self.c.backup_dbname_combo.selection_string() {
            let date = chrono::offset::Local::now();
            let date_st = date.format("%Y%m%d_%H%M%S");
            let filename = format!("{}_{}.zip", name, date_st);
            self.c.backup_filename_input.set_text(&filename);
        }
    }

    fn set_dbnames(&mut self, dbnames_all: &Vec<String>, bbf_db: &str) {
        let mut dbnames: Vec<String> = dbnames_all.iter().filter(|name| {
            !vec!("master", "msdb", "tempdb").contains(&name.as_str())
        }).map(|name| name.clone()).collect();
        dbnames.sort();
        self.c.backup_dbname_combo.set_collection(dbnames);
        self.c.backup_dbname_combo.set_selection(Some(0));
        self.on_dbname_changed(nwg::EventData::NoData);
        self.c.restore_bbf_db_input.set_text(bbf_db);
    }

    fn set_status_bar_dbconn_label(&self, text: &str) {
        self.c.status_bar.set_text(0, &format!("  DB connection: {}", text));
    }
}
