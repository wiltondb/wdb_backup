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

use super::*;
use nwg::EventData;

#[derive(Default)]
pub struct ConnectDialog {
    pub(super) c: ConnectDialogControls,

    args: ConnectDialogArgs,
    result: ConnectDialogResult,
    check_join_handle: ui::PopupJoinHandle<ConnectCheckDialogResult>,
    load_join_handle: ui::PopupJoinHandle<LoadDbnamesDialogResult>,
}

impl ConnectDialog {
    pub(super) fn open_check_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let config = self.config_from_input();
        let args = ConnectCheckDialogArgs::new(&self.c.check_notice, config);
        self.check_join_handle = ConnectCheckDialog::popup(args);
    }

    pub(super) fn await_check_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.check_notice.receive();
        let _ = self.check_join_handle.join();
        ui::shake_window(&self.c.window);
        self.c.update_tab_order();
    }

    pub(super) fn open_load_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let config = self.config_from_input();
        let args = LoadDbnamesDialogArgs::new(&self.c.load_notice, config);
        self.load_join_handle = LoadDbnamesDialog::popup(args);
    }

    pub(super) fn await_load_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.load_notice.receive();
        let res = self.load_join_handle.join();
        if !res.success {
            ui::shake_window(&self.c.window);
            self.c.update_tab_order();
        } else {
            let config = self.config_from_input();
            self.result = ConnectDialogResult::new(config, res.dbnames, res.bbf_db);
            self.close(nwg::EventData::NoData);
        }
    }

    pub(super) fn on_use_pgpass_checkbox_changed(&mut self, _: nwg::EventData) {
        if self.c.use_pgpass_checkbox.check_state() == nwg::CheckBoxState::Checked {
            self.c.password_input.set_readonly(true);
        } else {
            self.c.password_input.set_readonly(false);
        }
    }

    pub(super) fn on_port_input_changed(&mut self, _: nwg::EventData) {
        self.correct_port_value();
    }

    pub(super) fn on_enable_tls_checkbox_changed(&mut self, _: nwg::EventData) {
        self.sync_tls_checkboxes_state();
    }

    fn correct_port_value(&self) {
        let text = self.c.port_input.text();
        if text.len() == 0 {
            self.c.port_input.set_text("1");
            return;
        }
        let num = match text.parse::<u128>() {
            Err(_) => {
                self.c.port_input.set_text("5432");
                return;
            },
            Ok(n) => n
        };
        if num > 65535 {
            self.c.port_input.set_text("65535");
        }
    }

    fn config_from_input(&self) -> PgConnConfig {
        let port = match self.c.port_input.text().parse::<u16>() {
            Ok(n) => n,
            Err(_) => 5432,
        };
        PgConnConfig {
            hostname: self.c.hostname_input.text(),
            port,
            username: self.c.username_input.text(),
            password: self.c.password_input.text(),
            use_pgpass_file: self.c.use_pgpass_checkbox.check_state() == nwg::CheckBoxState::Checked,
            connect_db: self.c.connect_db_input.text(),
            enable_tls: self.c.enable_tls_checkbox.check_state() == nwg::CheckBoxState::Checked,
            accept_invalid_tls: self.c.enable_tls_checkbox.enabled() &&
                self.c.accept_invalid_tls_checkbox.check_state() == nwg::CheckBoxState::Checked
        }
    }

    fn config_to_input(&self, config: &PgConnConfig) {
        self.c.hostname_input.set_text(&config.hostname);
        self.c.port_input.set_text(&config.port.to_string());
        self.c.username_input.set_text(&config.username);
        self.c.password_input.set_text(&config.password);
        let pgpass_state = if config.use_pgpass_file {
            self.c.password_input.set_readonly(true);
            nwg::CheckBoxState::Checked
        } else {
            self.c.password_input.set_readonly(false);
            nwg::CheckBoxState::Unchecked
        };
        self.c.use_pgpass_checkbox.set_check_state(pgpass_state);
        self.c.connect_db_input.set_text(&config.connect_db);
        let tls_state = if config.enable_tls {
            nwg::CheckBoxState::Checked
        } else {
            nwg::CheckBoxState::Unchecked
        };
        self.c.enable_tls_checkbox.set_check_state(tls_state);
        let accept_state = if config.accept_invalid_tls {
            nwg::CheckBoxState::Checked
        } else {
            nwg::CheckBoxState::Unchecked
        };
        self.c.accept_invalid_tls_checkbox.set_check_state(accept_state);
    }

    fn sync_tls_checkboxes_state(&self) {
        let enabled = self.c.enable_tls_checkbox.check_state() == nwg::CheckBoxState::Checked;
        self.c.accept_invalid_tls_checkbox.set_enabled(enabled);
    }
}

impl ui::PopupDialog<ConnectDialogArgs, ConnectDialogResult> for ConnectDialog {
    fn popup(args: ConnectDialogArgs) -> ui::PopupJoinHandle<ConnectDialogResult> {
        let join_handle = thread::spawn(move || {
            let data = Self {
                args,
                ..Default::default()
            };
            let mut dialog = Self::build_ui(data).expect("Failed to build UI");
            nwg::dispatch_thread_events();
            dialog.result()
        });
        ui::PopupJoinHandle::from(join_handle)
    }

    fn init(&mut self) {
        self.config_to_input(&self.args.pg_conn_config);
        self.result = ConnectDialogResult::cancelled();
        ui::shake_window(&self.c.window);
    }

    fn result(&mut self) -> ConnectDialogResult {
        self.result.clone()
    }

    fn close(&mut self, _: nwg::EventData) {
        self.args.notify_parent();
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&mut self, _: EventData) {
        self.c.update_tab_order();
    }
}
