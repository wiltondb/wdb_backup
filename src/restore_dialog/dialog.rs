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

use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::time;

use super::*;
use crate::restore_dialog::args::PgRestoreArgs;
use crate::common::PgAccessError;

#[derive(Default)]
pub struct RestoreDialog {
    pub(super) c: RestoreDialogControls,

    args: RestoreDialogArgs,
    command_join_handle: ui::PopupJoinHandle<RestoreResult>,
    dialog_result: RestoreDialogResult,

    progress_pending: Vec<String>,
    progress_last_updated: u128,
}

impl RestoreDialog {

    pub(super) fn on_progress(&mut self, _: nwg::EventData) {
        let msg = self.c.progress_notice.receive();
        self.progress_pending.push(msg);
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis();
        if now - self.progress_last_updated > 100 {
            let joined = self.progress_pending.join("\r\n");
            self.progress_pending.clear();
            self.progress_last_updated = now;
            self.c.details_box.appendln(&joined);
        }
    }

    pub(super) fn on_complete(&mut self, _: nwg::EventData) {
        self.c.complete_notice.receive();
        let res = self.command_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = RestoreDialogResult::failure();
            self.c.label.set_text("Restore failed");
            self.progress_pending.push(res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = RestoreDialogResult::success();
            self.c.label.set_text("Restore complete");
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        }
        if self.progress_pending.len() > 0 {
            let joined = self.progress_pending.join("\r\n");
            self.c.details_box.appendln(&joined);
            self.progress_pending.clear();
        }
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

    fn unzip_file(progress: &ui::SyncNoticeValueSender<String>, zipfile: &str) -> Result<String, io::Error> {
        let file_path = Path::new(zipfile);
        let parent_dir = match file_path.parent() {
            Some(dir) => dir,
            None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Error accessing parent directory")))
        };
        let parent_dir_st = match parent_dir.to_str() {
            Some(st) => st,
            None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Error reading parent directory name")))
        };
        let listener = |en: &str| {
            progress.send_value(en);
        };
        match unzip_directory(zipfile, parent_dir_st, &listener) {
            Ok(dirname) => {
                let dir_path = parent_dir.join(Path::new(&dirname));
                match dir_path.to_str() {
                    Some(st) => Ok(st.to_string()),
                    None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                        "Error reading dest directory name")))
                }
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Unzip error, file: {}, message: {}", zipfile, e)))
        }
    }

    fn check_db_does_not_exist(pg_conn_config: &PgConnConfig, dbname: &str) -> Result<(), PgAccessError> {
        let mut client = pg_conn_config.open_connection()?;
        let cursor = client.query("select name from sys.babelfish_sysdatabases", &[])?;
        for row in cursor.iter() {
            let name: String = row.get("name");
            if name.to_lowercase() == dbname.to_lowercase() {
                return Err(PgAccessError::from_string(format!("Database with name '{}' already exists", dbname)))
            }
        };
        client.close()?;
        Ok(())
    }

    fn create_role_if_not_exist(client: &mut postgres::Client, dbname: &str, role: &str) -> Result<(), PgAccessError> {
        let rolname = format!("{}_{}", dbname, role);
        let list = client.query("select (count(1) > 0) as role_exist from pg_catalog.pg_roles where rolname = $1", &[&rolname])?;
        let exists: bool = list[0].get(0);
        if !exists {
            client.execute(&format!("CREATE ROLE {}", rolname), &[])?;
            client.execute(&format!("ALTER ROLE {} WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", rolname), &[])?;
        }
        Ok(())
    }

    fn restore_global_data(pcc: &PgConnConfig, dbname: &str) -> Result<(), PgAccessError> {
        let mut client = pcc.open_connection()?;
        Self::create_role_if_not_exist(&mut client, dbname, "db_owner")?;
        Self::create_role_if_not_exist(&mut client, dbname, "dbo")?;
        Self::create_role_if_not_exist(&mut client, dbname, "guest")?;
        client.execute(&format!("GRANT {}_db_owner TO {}_dbo GRANTED BY sysadmin", dbname, dbname), &[])?;
        client.execute(&format!("GRANT {}_dbo TO sysadmin GRANTED BY sysadmin", dbname), &[])?;
        client.execute(&format!("GRANT {}_guest TO sysadmin GRANTED BY sysadmin", dbname), &[])?;
        client.execute(&format!("GRANT {}_guest TO {}_db_owner GRANTED BY sysadmin", dbname, dbname), &[])?;
        client.close()?;
        Ok(())
    }

    fn run_pg_restore(progress: &ui::SyncNoticeValueSender<String>, pcc: &PgConnConfig, dir: &str, bbf_db: &str) -> Result<(), io::Error> {
        let cur_exe = env::current_exe()?;
        let _bin_dir = match cur_exe.parent() {
            Some(path) => path,
            None => { // cannot happen
                let exe_st = cur_exe.to_str().unwrap_or("");
                return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "Parent dir failure, exe path: {}", exe_st)))
            }
        };
        // todo
        //let pg_restore_exe = bin_dir.as_path().join("pg_restore.exe");
        let pg_restore_exe = Path::new("C:\\Program Files\\WiltonDB Software\\wiltondb3.3\\bin\\pg_restore.exe").to_path_buf();
        let cmd = duct::cmd!(
            pg_restore_exe,
            "-v",
            "-h", &pcc.hostname,
            "-p", &pcc.port.to_string(),
            "-U", &pcc.username,
            "-d", bbf_db,
            "-F", "d",
            "-j", "1",
            dir
        )
            .env("PGPASSWORD", &pcc.password)
            .before_spawn(|pcmd| {
                // create no window
                let _ = pcmd.creation_flags(0x08000000);
                Ok(())
            });
        let reader = cmd.stderr_to_stdout().reader()?;
        for line in BufReader::new(&reader).lines() {
            match line {
                Ok(ln) => progress.send_value(ln),
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "pg_restore process failure: {}", e)))
            }
        };
        match reader.try_wait() {
            Ok(opt) => match opt {
                Some(_) => { },
                None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "pg_restore process failure")))
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "pg_restore process failure: {}", e)))
        }

        Ok(())
    }

    fn run_restore(progress: &ui::SyncNoticeValueSender<String>, pcc: &PgConnConfig, ra: &PgRestoreArgs) -> RestoreResult {
        progress.send_value(format!("Running restore into DB: {} ...", ra.dest_db_name));

        // db check
        if let Err(e) = Self::check_db_does_not_exist(pcc, &ra.dest_db_name) {
            return RestoreResult::failure(format!("{}", e))
        }

        // unzip
        progress.send_value(format!("Unzipping file: {} ...", &ra.zip_file_path));
        let dir = match Self::unzip_file(progress, &ra.zip_file_path) {
            Ok(dir) => dir,
            Err(e) => return RestoreResult::failure(format!("{}", e))
        };

        // rewrite
        progress.send_value("Updating DB name ...");
        if let Err(e) = rewrite_toc(&dir, &ra.dest_db_name) {
            return RestoreResult::failure(format!("{}", e))
        }

        // global data
        progress.send_value("Restoring roles ...");
        if let Err(e) = Self::restore_global_data(pcc, &ra.dest_db_name) {
            return RestoreResult::failure(format!("{}", e))
        }

        // run restore
        progress.send_value("Running pg_restore ...");
        if let Err(e) = Self::run_pg_restore(progress, pcc, &dir, &ra.bbf_db_name) {
            return RestoreResult::failure(format!("{}", e))
        };

        // clean up
        progress.send_value("Cleaning up temp directory ...");
        if let Err(e) = fs::remove_dir_all(Path::new(&dir)) {
            return RestoreResult::failure(format!("{}", e))
        };

        progress.send_value("Restore complete");
        RestoreResult::success()
    }
}

impl ui::PopupDialog<RestoreDialogArgs, RestoreDialogResult> for RestoreDialog {
    fn popup(args: RestoreDialogArgs) -> ui::PopupJoinHandle<RestoreDialogResult> {
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
        let complete_sender = self.c.complete_notice.sender();
        let progress_sender = self.c.progress_notice.sender();
        let pcc: PgConnConfig = self.args.pg_conn_config.clone();
        let pra: PgRestoreArgs = self.args.pg_restore_args.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = RestoreDialog::run_restore(&progress_sender, &pcc, &pra);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            complete_sender.send();
            res
        });
        self.command_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> RestoreDialogResult {
        self.dialog_result.clone()
    }

    fn close(&mut self, _: nwg::EventData) {
        self.args.send_notice();
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }
}

