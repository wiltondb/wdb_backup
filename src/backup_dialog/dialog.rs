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
use std::path::Path;

use super::*;
use crate::backup_dialog::args::PgDumpArgs;

#[derive(Default)]
pub struct BackupDialog {
    pub(super) c: BackupDialogControls,

    args: BackupDialogArgs,
    command_join_handle: ui::PopupJoinHandle<BackupResult>,
    dialog_result: BackupDialogResult
}

impl BackupDialog {
    pub(super) fn on_command_complete(&mut self, _: nwg::EventData) {
        self.c.command_notice.receive();
        let res = self.command_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = BackupDialogResult::failure();
            self.c.label.set_text("Backup failed");
            self.c.details_box.set_text(&res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = BackupDialogResult::success();
            self.c.label.set_text("Backup complete");
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

    fn run_command(pcc: &PgConnConfig, dbname: &str, dest_dir: &str) -> Result<String, io::Error> {
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
        //let pg_dump_exe = bin_dir.as_path().join("pg_dump.exe");
        let pg_dump_exe = Path::new("C:\\projects\\postgres\\dist\\bin\\pg_dump.exe").to_path_buf();
        env::set_var("PGPASSWORD", &pcc.password);
        let create_no_window: u32 = 0x08000000;
        println!("dest_dir: {}", &dest_dir);
        match process::Command::new(pg_dump_exe.as_os_str())
            .arg("-v")
            .arg("-h").arg(&pcc.hostname)
            .arg("-p").arg(&pcc.port.to_string())
            .arg("-U").arg(&pcc.username)
            .arg("--bbf-database-name").arg(&dbname)
            .arg("-Fd")
            //.arg("-Z6")
            .arg("-Z0")
            .arg("-f").arg(&dest_dir)
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
                        "Backup error, status code: {}\r\n\r\nstderr: {}\r\nstdout: {}", code, stderr, stdout)))
                }
            },
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!(
                "Backup spawn error: {}", e)))
        }
    }

    fn zip_dest_directory(dest_dir: &str, filename: &str) -> Result<(), io::Error> {
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
        println!("dest_file_st: {}", dest_file_st);
        match zip_directory(dest_dir_st, dest_file_st, 0) {
            Ok(_) => {},
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
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

    fn run_backup(pcc: &PgConnConfig, pargs: &PgDumpArgs) -> BackupResult {
        // ensure no dest dir
        let (dest_dir, filename) = match Self::prepare_dest_dir(&pargs.parent_dir, &pargs.dest_filename) {
            Ok(st) => st,
            Err(e) => return BackupResult::failure(e.to_string())
        };
        // spawn and wait
        match BackupDialog::run_command(pcc, &pargs.dbname, &dest_dir) {
            Ok(output) => {
                // zip results
                match Self::zip_dest_directory(&dest_dir, &filename) {
                    Ok(_) => BackupResult::success(output),
                    Err(e) => return BackupResult::failure(format!(
                        "Error zipping destination directory, path: {}, error: {}", &dest_dir, e))
                }
            },
            Err(e) => BackupResult::failure(e.to_string())
        }
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
        let sender = self.c.command_notice.sender();
        let pcc: PgConnConfig = self.args.pg_conn_config.clone();
        let pargs = self.args.pg_dump_args.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = BackupDialog::run_backup(&pcc, &pargs);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            sender.send();
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

