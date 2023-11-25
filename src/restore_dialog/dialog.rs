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
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

use super::*;
use crate::restore_dialog::args::PgRestoreArgs;
use crate::common::PgAccessError;

#[derive(Default)]
pub struct RestoreDialog {
    pub(super) c: RestoreDialogControls,

    args: RestoreDialogArgs,
    command_join_handle: ui::PopupJoinHandle<RestoreResult>,
    dialog_result: RestoreDialogResult
}

impl RestoreDialog {
    pub(super) fn on_command_complete(&mut self, _: nwg::EventData) {
        self.c.command_notice.receive();
        let res = self.command_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = RestoreDialogResult::failure();
            self.c.label.set_text("Restore failed");
            self.c.details_box.set_text(&res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = RestoreDialogResult::success();
            self.c.label.set_text("Restore complete");
            self.c.details_box.set_text(&res.output);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
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

    fn unzip_file(zipfile: &str) -> Result<String, io::Error> {
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
        match unzip_directory(zipfile, parent_dir_st) {
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

    fn restore_global_data(pcc: &PgConnConfig, dbname: &str) -> Result<(), PgAccessError> {
        println!("{}", dbname);
        let mut client = pcc.open_connection()?;
        // todo: error handling
        client.execute(&format!("CREATE ROLE {}_db_owner", dbname), &[])?;
        client.execute(&format!("ALTER ROLE {}_db_owner WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", dbname), &[])?;
        client.execute(&format!("CREATE ROLE {}_dbo", dbname), &[])?;
        client.execute(&format!("ALTER ROLE {}_dbo WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", dbname), &[])?;
        client.execute(&format!("CREATE ROLE {}_guest", dbname), &[])?;
        client.execute(&format!("ALTER ROLE {}_guest WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", dbname), &[])?;
        client.execute(&format!("GRANT {}_db_owner TO {}_dbo GRANTED BY sysadmin", dbname, dbname), &[])?;
        client.execute(&format!("GRANT {}_dbo TO sysadmin GRANTED BY sysadmin", dbname), &[])?;
        client.execute(&format!("GRANT {}_guest TO sysadmin GRANTED BY sysadmin", dbname), &[])?;
        client.execute(&format!("GRANT {}_guest TO {}_db_owner GRANTED BY sysadmin", dbname, dbname), &[])?;
        client.close()?;
        Ok(())
    }

    fn run_pg_restore(pcc: &PgConnConfig, dir: &str, bbf_db: &str) -> Result<String, io::Error> {
        let cur_exe = env::current_exe()?;
        let bin_dir = match cur_exe.parent() {
            Some(path) => path,
            None => { // cannot happen
                let exe_st = cur_exe.to_str().unwrap_or("");
                return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "Parent dir failure, exe path: {}", exe_st)))
            }
        };
        // todo
        //let pg_restore_exe = bin_dir.as_path().join("pg_restore.exe");
        let pg_restore_exe = Path::new("C:\\projects\\postgres\\dist\\bin\\pg_restore.exe").to_path_buf();
        env::set_var("PGPASSWORD", &pcc.password);
        let create_no_window: u32 = 0x08000000;
        match process::Command::new(pg_restore_exe.as_os_str())
            .arg("-v")
            .arg("-h").arg(&pcc.hostname)
            .arg("-p").arg(&pcc.port.to_string())
            .arg("-U").arg(&pcc.username)
            .arg("-d").arg(bbf_db)
            .arg("-Fd")
            .arg(dir)
            .creation_flags(create_no_window)
            .output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout[..]).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr[..]).to_string();
                if output.status.success() {
                    if !stdout.is_empty() || ! stderr.is_empty() {
                        Ok(format!("{}\n{}", stdout, stderr))
                    } else {
                        Ok(format!("{}{}", stdout, stderr))
                    }
                } else {
                    let code = match output.status.code() {
                        Some(code) => code,
                        None => -1
                    };
                    Err(io::Error::new(io::ErrorKind::Other, format!(
                        "Restore error, status code: {}\r\n\r\nstderr: {}\r\nstdout: {}", code, stderr, stdout)))
                }
            },
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!(
                "Restore spawn error: {}", e)))
        }
    }

    fn run_restore(pcc: &PgConnConfig, ra: &PgRestoreArgs) -> RestoreResult {
        let dir = match Self::unzip_file(&ra.zip_file_path) {
            Ok(dir) => dir,
            Err(e) => return RestoreResult::failure(format!("{}", e))
        };
        match rewrite_toc(&dir, &ra.dest_db_name) {
            Ok(_) => {},
            Err(e) => return RestoreResult::failure(format!("{}", e))
        }
        match  Self::restore_global_data(pcc, &ra.dest_db_name) {
            Ok(_) => {},
            Err(e) => return RestoreResult::failure(format!("{}", e))
        }
        match Self::run_pg_restore(pcc, &dir, &ra.bbf_db_name) {
            Ok(output) => RestoreResult::success(output),
            Err(e) => return RestoreResult::failure(format!("{}", e))
        }
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
        let sender = self.c.command_notice.sender();
        let pcc: PgConnConfig = self.args.pg_conn_config.clone();
        let pra: PgRestoreArgs = self.args.pg_restore_args.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = RestoreDialog::run_restore(&pcc, &pra);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            sender.send();
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

