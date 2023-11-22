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
pub struct PgDumpArgs {
    pub(super) dbname: String,
    pub(super) parent_dir: String,
    pub(super) dest_filename: String,
}

#[derive(Default)]
pub struct BackupDialogArgs {
    pub(super) notice_sender:  ui::SyncNoticeSender,
    pub(super) pg_conn_config: PgConnConfig,
    pub(super) pg_dump_args: PgDumpArgs,
}

impl BackupDialogArgs {
    pub fn new(notice: &ui::SyncNotice, pg_conn_config: &PgConnConfig, dbname: &str, parent_dir: &str, dest_filename: &str) -> Self {
        Self {
            notice_sender: notice.sender(),
            pg_conn_config: pg_conn_config.clone(),
            pg_dump_args: PgDumpArgs {
                dbname: dbname.to_string(),
                parent_dir: parent_dir.to_string(),
                dest_filename: dest_filename.to_string()
            },
        }
    }

    pub fn send_notice(&self) {
        self.notice_sender.send()
    }
}

impl ui::PopupArgs for BackupDialogArgs {
    fn notify_parent(&self) {
        self.notice_sender.send()
    }
}
