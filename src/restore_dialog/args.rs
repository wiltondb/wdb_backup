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


#[derive(Default, Clone)]
pub struct PgRestoreArgs {
    pub(super) zip_file_path: String,
    pub(super) dest_db_name: String,
    pub(super) bbf_db_name: String,
}

#[derive(Default)]
pub struct RestoreDialogArgs {
    pub(super) notice_sender:  ui::SyncNoticeSender,
    pub(super) pg_conn_config: PgConnConfig,
    pub(super) pg_restore_args: PgRestoreArgs,
}

impl RestoreDialogArgs {
    pub fn new(notice: &ui::SyncNotice, pg_conn_config: &PgConnConfig,
               zip_file_path: &str, dest_db_name: &str, bbf_db_name: &str) -> Self {
        Self {
            notice_sender: notice.sender(),
            pg_conn_config: pg_conn_config.clone(),
            pg_restore_args: PgRestoreArgs {
                zip_file_path: zip_file_path.to_string(),
                dest_db_name: dest_db_name.to_string(),
                bbf_db_name: bbf_db_name.to_string(),
            }
        }
    }

    pub fn send_notice(&self) {
        self.notice_sender.send()
    }
}

impl ui::PopupArgs for RestoreDialogArgs {
    fn notify_parent(&self) {
        self.notice_sender.send()
    }
}
