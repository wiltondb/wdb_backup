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

use std::os::windows::process::CommandExt;
use std::process::Command;
use std::process::Stdio;

use super::*;

#[derive(Default)]
pub struct AppWindow {
    pub(super) c: AppWindowControls,

    pg_conn_config: PgConnConfig,

    about_dialog_join_handle: ui::PopupJoinHandle<()>,
    connect_dialog_join_handle: ui::PopupJoinHandle<ConnectDialogResult>,
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
        //self.c.filter_input.set_enabled(true);
    }

    pub(super) fn open_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = ConnectDialogArgs::new(&self.c.connect_notice, self.pg_conn_config.clone());
        self.connect_dialog_join_handle = ConnectDialog::popup(args);
    }

    pub(super) fn await_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.connect_notice.receive();
        let _ = self.connect_dialog_join_handle.join();
        //self.c.filter_input.set_enabled(true);
    }

    pub(super) fn open_website(&mut self, _: nwg::EventData) {
        let create_no_window: u32 = 0x08000000;
        let _ = Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("https://wiltondb.com")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(create_no_window)
            .status();
    }

    pub(super) fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }

    fn set_status_bar_dbconn_label(&self, text: &str) {
        self.c.status_bar.set_text(0, &format!("  DB connection: {}", text));
    }

    pub(super) fn choose_dest_dir(&mut self, _: nwg::EventData) {
        if let Ok(d) = std::env::current_dir() {
            if let Some(d) = d.to_str() {
                self.c.dest_dir_chooser.set_default_folder(d).expect("Failed to set default folder.");
            }
        }

        if self.c.dest_dir_chooser.run(Some(&self.c.window)) {
            self.c.dest_dir_input.set_text("");
            if let Ok(directory) = self.c.dest_dir_chooser.get_selected_item() {
                let dir = directory.into_string().unwrap();
                self.c.dest_dir_input.set_text(&dir);
            }
        }
    }
}
