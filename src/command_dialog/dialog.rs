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
use std::os::windows::process::CommandExt;
use std::path::Path;

use super::*;

#[derive(Default)]
pub struct CommandDialog {
    pub(super) c: CommandDialogControls,

    args: CommandDialogArgs,
    command_join_handle: ui::PopupJoinHandle<CommandResult>,
    dialog_result: CommandDialogResult
}

impl CommandDialog {
    pub(super) fn on_command_complete(&mut self, _: nwg::EventData) {
        self.c.command_notice.receive();
        let res = self.command_join_handle.join();
        let success = res.error.is_empty();
        self.stop_progress_bar(success.clone());
        if !success {
            self.dialog_result = CommandDialogResult::failure();
            self.c.label.set_text("Command failed");
            self.c.details_box.set_text(&res.error);
            self.c.copy_clipboard_button.set_enabled(true);
            self.c.close_button.set_enabled(true);
        } else {
            self.dialog_result = CommandDialogResult::success();
            self.c.label.set_text("Command complete");
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

    fn run_command(command: &PgCommand) -> Result<String, PgAccessError> {
        // todo: failures
        if !command.sql_statements.is_empty() {
            let mut client = command.conn_config.open_connection()?;
            for sql in &command.sql_statements {
                client.execute(sql, &[])?;
            }
            client.close()?;
        }
        let create_no_window: u32 = 0x08000000;
        // todo: unset
        for (name, value) in &command.env_vars {
            env::set_var(&name, &value);
        }
        let mut cmd = process::Command::new(&command.program);
        for a in &command.args {
            cmd.arg(a);
        }
        cmd.creation_flags(create_no_window);
        match cmd.output() {
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
                    Err(PgAccessError::from_string(format!(
                        "Command error, status code: {}\r\n\r\nstderr: {}\r\nstdout: {}", code, stderr, stdout)))
                }
            },
            Err(e) => Err(PgAccessError::from_string(format!("Command spawn error: {}", e)))
        }
    }

    fn ensure_no_dest_dir(dest_dir: &str) -> Result<(), io::Error> {
        let dir_path = Path::new(dest_dir);
        let _ = fs::remove_dir_all(&dir_path);
        if dir_path.exists() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error removing directory: {}", dest_dir)));
        }
        Ok(())
    }

    fn zip_dest_directory(zd: &PgCommandZip) -> Result<(), io::Error> {
        let dest_dir_path = Path::new(&zd.dest_dir);
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
        let parent_path_buf = parent_path.join(&zd.zip_file_name);
        let dest_file_st = match parent_path_buf.to_str() {
            Some(st) => st,
            None => return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!(
                "Error accessing destination file")))
        };
        match zip_directory(dest_dir_st, dest_file_st, zd.comp_level) {
            Ok(_) => {},
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
        std::fs::remove_dir_all(dest_dir_path)?;
        Ok(())
    }

    fn process_command(cmd: PgCommand) -> CommandResult {
        // ensure no dest dir
        if !cmd.ensure_dest_dir.is_empty() {
            match Self::ensure_no_dest_dir(&cmd.ensure_dest_dir) {
                Ok(_) => {},
                Err(e) => return CommandResult::failure(e.to_string())
            }
        }
        // spawn and wait
        let res = match CommandDialog::run_command(&cmd) {
            Ok(output) => CommandResult::success(output),
            Err(e) => CommandResult::failure(e.to_string())
        };
        // zip results
        if res.error.is_empty() && cmd.zip_dest_dir.enabled {
            match Self::zip_dest_directory(&cmd.zip_dest_dir) {
                Ok(_) => {},
                Err(e) => return CommandResult::failure(format!(
                    "Error zipping destination directory, path: {}, error: {}", cmd.zip_dest_dir.dest_dir, e))
            }
        }
        res
    }
}

impl ui::PopupDialog<CommandDialogArgs, CommandDialogResult> for CommandDialog {
    fn popup(args: CommandDialogArgs) -> ui::PopupJoinHandle<CommandDialogResult> {
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
        let cmd = self.args.command.clone();
        let join_handle = thread::spawn(move || {
            let start = Instant::now();
            let res = CommandDialog::process_command(cmd);
            let remaining = 1000 - start.elapsed().as_millis() as i64;
            if remaining > 0 {
                thread::sleep(Duration::from_millis(remaining as u64));
            }
            sender.send();
            res
        });
        self.command_join_handle = ui::PopupJoinHandle::from(join_handle);
    }

    fn result(&mut self) -> CommandDialogResult {
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

