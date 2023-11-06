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
pub struct ConnectCheckDialog {
    pub(super) c: ConnectCheckDialogControls,

    args: ConnectCheckDialogArgs,
    check_join_handle: ui::PopupJoinHandle<ConnectCheckResult>,
}

impl ConnectCheckDialog {
    pub(super) fn on_connection_check_complete(&mut self, _: nwg::EventData) {
        self.c.check_notice.receive();
        let res = self.check_join_handle.join();
        self.stop_progress_bar(res.success);
        let label = if res.success {
            "Connection successful"
        } else {
            "Connection failed"
        };
        self.c.label.set_text(label);
        self.c.details_box.set_text(&res.message);
        self.c.copy_clipboard_button.set_enabled(true);
        self.c.close_button.set_enabled(true);
    }

    pub(super) fn copy_to_clipboard(&mut self, _: nwg::EventData) {
        let text = self.c.details_box.text();
        let _ = set_clipboard(formats::Unicode, &text);
    }

    fn stop_progress_bar(&self, success: bool) {
        self.c.progress_bar.set_marquee(false, 0);
        self.c.progress_bar.remove_flags(nwg::ProgressBarFlags::MARQUEE);
        self.c.progress_bar.set_pos(1);
        if !success {
            self.c.progress_bar.set_state(nwg::ProgressBarState::Error)
        }
    }

    fn check_postgres_conn(pg_conn_config: &PgConnConfig) -> Result<String, PgAccessError> {
        let mut client = pg_conn_config.open_connection()?;
        let vec = client.query("select version()", &[])?;
        let row = &vec[0];
        let res: String = row.get("version");
        client.close()?;
        Ok(res)
    }
}

impl ui::PopupDialog<ConnectCheckDialogArgs, ConnectCheckDialogResult> for ConnectCheckDialog {
    fn popup(args: ConnectCheckDialogArgs) -> ui::PopupJoinHandle<ConnectCheckDialogResult> {
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
        let sender = self.c.check_notice.sender();
        let pgconf = self.args.pg_conn_config.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = match ConnectCheckDialog::check_postgres_conn(&pgconf) {
                Ok(version) => ConnectCheckResult::success(version),
                Err(e) => ConnectCheckResult::failure(format!("{}", e))
            };
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            sender.send();
            res
        });
        self.check_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> ConnectCheckDialogResult {
        // todo
        Default::default()
    }

    fn close(&mut self, _: nwg::EventData) {
        self.args.send_notice();
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&mut self, _: EventData) {
        self.c.update_tab_order();
    }
}

