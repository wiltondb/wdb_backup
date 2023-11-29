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
use std::io::BufRead;
use std::io::BufReader;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::time;

use super::*;
use crate::backup_dialog::args::PgDumpArgs;

#[derive(Default)]
pub struct BackupDialog {
    pub(super) c: BackupDialogControls,

    args: BackupDialogArgs,
    command_join_handle: ui::PopupJoinHandle<BackupResult>,
    dialog_result: BackupDialogResult,

    progress_pending: Vec<String>,
    progress_last_updated: u128,
}

impl BackupDialog {

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
            self.dialog_result = BackupDialogResult::failure();
            self.c.label.set_text("Backup failed");
            self.progress_pending.push(res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = BackupDialogResult::success();
            self.c.label.set_text("Backup complete");
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

    fn run_command(progress: &ui::SyncNoticeValueSender<String>, pcc: &PgConnConfig, dbname: &str, dest_dir: &str) -> Result<(), io::Error> {
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
        //let pg_dump_exe = bin_dir.as_path().join("pg_dump.exe");
        let pg_dump_exe = Path::new("C:\\Program Files\\WiltonDB Software\\wiltondb3.3\\bin\\pg_dump.exe").to_path_buf();
        env::set_var("PGPASSWORD", &pcc.password);
        let cmd = duct::cmd!(
            pg_dump_exe,
            "-v",
            "-h", &pcc.hostname,
            "-p", &pcc.port.to_string(),
            "-U", &pcc.username,
            "--bbf-database-name", &dbname,
            "-F", "d",
            "-Z", "6",
            "-j", "4",
            "-f", &dest_dir
        ).before_spawn(|pcmd| {
            // create no window
            let _ = pcmd.creation_flags(0x08000000);
            Ok(())
        });
        let reader = cmd.stderr_to_stdout().reader()?;
        for line in BufReader::new(&reader).lines() {
            match line {
                Ok(ln) => progress.send_value(ln),
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                    "pg_dump process failure: {}", e)))
            }
        };
        match reader.try_wait() {
            Ok(opt) => match opt {
                Some(_) => { },
                None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                        "pg_dump process failure")))
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "pg_dump process failure: {}", e)))
        }

        Ok(())
    }

    fn zip_dest_directory(progress: &ui::SyncNoticeValueSender<String>, dest_dir: &str, filename: &str) -> Result<(), io::Error> {
        let dest_dir_path = Path::new(dest_dir);
        let parent_path = match dest_dir_path.parent() {
            Some(path) => path,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination directory parent")))
        };
        let dest_dir_st = match dest_dir_path.to_str() {
            Some(st) => st,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination directory")))
        };
        let dest_file_buf = parent_path.join(filename);
        let dest_file_st = match dest_file_buf.to_str() {
            Some(st) => st,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination file")))
        };
        let listener = |en: &str| {
            progress.send_value(en);
        };
        match zip_directory(dest_dir_st, dest_file_st, 0, &listener) {
            Ok(_) => {},
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        };
        std::fs::remove_dir_all(dest_dir_path)?;
        Ok(())
    }

    fn prepare_dest_dir(dest_parent_dir: &str, dest_filename: &str) -> Result<(String, String), io::Error> {
        let mut ext = Path::new(dest_filename).extension().unwrap_or(OsStr::new(""))
            .to_str().unwrap_or("").to_string();
        let mut filename = dest_filename.to_string();
        if ext.is_empty() {
            ext = "zip".to_string();
            filename = format!("{}.{}", filename, ext);
        }
        let dirname: String = filename.chars().take(filename.len() - (ext.len() + 1)).collect();
        let parent_dir_path = Path::new(dest_parent_dir);
        let dir_path = parent_dir_path.join(dirname);
        let dir_path_st = match dir_path.to_str() {
            Some(st) => st.to_string(),
            None => return Err(io::Error::new(io::ErrorKind::Other, format!(
                "Error reading directory name")))
        };
        let _ = fs::remove_dir_all(&dir_path);
        if dir_path.exists() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error removing directory: {}", dir_path_st)));
        }
        Ok((dir_path_st, filename))
    }

    fn run_backup(progress: &ui::SyncNoticeValueSender<String>, pcc: &PgConnConfig, pargs: &PgDumpArgs) -> BackupResult {
        progress.send_value("Running backup ...");

        // ensure no dest dir
        let (dest_dir, filename) = match Self::prepare_dest_dir(&pargs.parent_dir, &pargs.dest_filename) {
            Ok(tup) => tup,
            Err(e) => return BackupResult::failure(e.to_string())
        };
        let dest_file = Path::new(&pargs.parent_dir).join(Path::new(&filename)).to_string_lossy().to_string();
        progress.send_value(format!("Backup file: {}", dest_file));

        // spawn and wait
        progress.send_value("Running pg_dump ....");
        match BackupDialog::run_command(progress, pcc, &pargs.dbname, &dest_dir) {
            Ok(_) => { },
            Err(e) => {
                return BackupResult::failure(e.to_string());
            }
        };

        // zip results
        progress.send_value("Zipping destination directory ....");
        match Self::zip_dest_directory(progress, &dest_dir, &filename) {
            Ok(_) => {},
            Err(e) => {
                return BackupResult::failure(format!(
                    "Error zipping destination directory, path: {}, error: {}", &dest_dir, e));
            }
        };

        progress.send_value("Backup complete");
        BackupResult::success()
    }
}

impl ui::PopupDialog<BackupDialogArgs, BackupDialogResult> for BackupDialog {
    fn popup(args: BackupDialogArgs) -> ui::PopupJoinHandle<BackupDialogResult> {
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
        let pargs = self.args.pg_dump_args.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = BackupDialog::run_backup(&progress_sender, &pcc, &pargs);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            complete_sender.send();
            res
        });
        self.command_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> BackupDialogResult {
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

