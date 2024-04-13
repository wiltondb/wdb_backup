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

use pgdump_toc_rewrite;

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
        match zip_recurse::unzip_directory_listen(zipfile, parent_dir_st, listener) {
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

    fn check_db_does_not_exist(pg_conn_config: &PgConnConfig, ra: &PgRestoreArgs) -> Result<(), PgAccessError> {
        let mut client = pg_conn_config.open_connection_to_db(&ra.bbf_db_name)?;
        let rs = client.query("select name from sys.babelfish_sysdatabases", &[])?;
        for row in rs.iter() {
            let name: String = row.get("name");
            if name.to_lowercase() == ra.dest_db_name.to_lowercase() {
                return Err(PgAccessError::from_string(format!("Database with name '{}' already exists", &name)))
            }
        };
        client.close()?;
        Ok(())
    }

    fn create_role_if_not_exist(client: &mut postgres::Client, dbname: &str, role: &str) -> Result<Option<String>, PgAccessError> {
        let rolname = format!("{}_{}", dbname, role);
        let rs = client.query("select (count(1) > 0) as role_exist from pg_catalog.pg_roles where rolname = $1", &[&rolname])?;
        let exists: bool = rs[0].get(0);
        if !exists {
            client.execute(&format!("CREATE ROLE {} WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", rolname), &[])?;
            // db error: ERROR: must be superuser to alter superuser roles or change superuser attribute
            // client.execute(&format!("ALTER ROLE {} WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", rolname), &[])?;
            Ok(Some(rolname))
        } else {
            Ok(None)
        }
    }

    fn restore_global_data(pcc: &PgConnConfig, ra: &PgRestoreArgs) -> Result<Vec<String>, PgAccessError> {
        let mut client = pcc.open_connection_to_db(&ra.bbf_db_name)?;
        let dbname = &ra.dest_db_name;
        let mut res = Vec::new();
        for role in vec!(
            "db_owner",
            "dbo",
            "guest"
        ) {
            if let Some(rolename) = Self::create_role_if_not_exist(&mut client, dbname, role)? {
                res.push(rolename);
            }
        }
        client.execute(&format!("GRANT {}_db_owner TO {}_dbo", dbname, dbname), &[])?;
        client.execute(&format!("GRANT {}_dbo TO sysadmin", dbname), &[])?;
        client.execute(&format!("GRANT {}_guest TO sysadmin", dbname), &[])?;
        client.execute(&format!("GRANT {}_guest TO {}_db_owner", dbname, dbname), &[])?;
        client.close()?;
        Ok(res)
    }

    fn drop_created_roles(pcc: &PgConnConfig, bbf_db: &str, roles: &Vec<String>) -> Result<(), PgAccessError> {
        let mut client = pcc.open_connection_to_db(bbf_db)?;
        for rolname in roles {
            client.execute(&format!("DROP ROLE {}", rolname), &[])?;
        }
        client.close()?;
        Ok(())
    }

    fn run_pg_restore(progress: &ui::SyncNoticeValueSender<String>, pcc: &PgConnConfig, dir: &str, bbf_db: &str) -> Result<(), io::Error> {
        let cur_exe = env::current_exe()?;
        let bin_dir = match cur_exe.parent() {
            Some(path) => path,
            None => { // cannot happen
                let exe_st = cur_exe.to_str().unwrap_or("");
                return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "Parent dir failure, exe path: {}", exe_st)))
            }
        };
        let pg_restore_exe = bin_dir.join("pg_restore.exe");
        let cmd = duct::cmd!(
            pg_restore_exe,
            "-v",
            "-h", &pcc.hostname,
            "-p", &pcc.port.to_string(),
            "-U", &pcc.username,
            "-d", bbf_db,
            "-F", "d",
            "-j", "1",
            "--single-transaction",
            dir
        )
            .env("PGPASSWORD", &pcc.password)
            .stdin_null()
            .stderr_to_stdout()
            .stdout_capture()
            .before_spawn(|pcmd| {
                // create no window
                let _ = pcmd.creation_flags(0x08000000);
                Ok(())
            });
        let reader = match cmd.reader() {
            Ok(reader) => reader,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "pg_restore process spawn failure: {}", e)))
        };
        let mut buf_reader = BufReader::new(&reader);
        loop {
            let mut buf = vec!();
            match buf_reader.read_until(b'\n', &mut buf) {
                Ok(len) => {
                    if 0 == len {
                        break;
                    }
                    if buf.len() >= 2 {
                        let ln = String::from_utf8_lossy(&buf[0..buf.len() - 2]);
                        progress.send_value(ln);
                    }
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "pg_restore process failure: {}", e)))
            };
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
        if let Err(e) = Self::check_db_does_not_exist(pcc, ra) {
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
        let toc_path = Path::new(&dir).join("toc.dat");
        if let Err(e) = pgdump_toc_rewrite::rewrite_toc(&toc_path, &ra.dest_db_name) {
            return RestoreResult::failure(format!("{}", e))
        }

        // global data
        progress.send_value("Restoring roles ...");
        let roles = match Self::restore_global_data(pcc, ra) {
            Ok(roles) => roles,
            Err(e) => return RestoreResult::failure(format!("{}", e))
        };

        // run restore
        progress.send_value("Running pg_restore ...");
        if let Err(e) = Self::run_pg_restore(progress, pcc, &dir, &ra.bbf_db_name) {
            if roles.len() > 0 {
                progress.send_value(format!(
                    "Error: restore failed, cleaning up global roles we created: {}", roles.join(", ")));
                match Self::drop_created_roles(pcc, &ra.bbf_db_name, &roles) {
                    Ok(_) => progress.send_value("Global roles cleanup complete"),
                    Err(e) => progress.send_value(format!(
                        "Error cleaning up global roles: {}", e))
                }
            }
            return RestoreResult::failure(format!("{}", e))
        };

        // clean up
        progress.send_value("Cleaning up temp directory ...");
        if let Err(e) = fs::remove_dir_all(Path::new(&dir)) {
            progress.send_value(format!(
                "Warning: error removing tem directory: {}, message: {}", dir, e));
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

