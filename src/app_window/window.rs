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

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use chrono;
use uuid::Uuid;

use super::*;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Default)]
pub struct AppWindow {
    pub(super) c: AppWindowControls,

    pg_conn_config: PgConnConfig,

    about_dialog_join_handle: ui::PopupJoinHandle<()>,
    connect_dialog_join_handle: ui::PopupJoinHandle<ConnectDialogResult>,
    command_dialog_join_handle: ui::PopupJoinHandle<CommandDialogResult>,
}

impl AppWindow {

    pub fn new() -> Self {
        Default::default()
    }

    pub(super) fn init(&mut self) {
        self.pg_conn_config.hostname = String::from("localhost");
        self.pg_conn_config.port = 5432;
        // todo: removeme
        self.pg_conn_config.username = String::from("wilton");
        self.pg_conn_config.password = String::from("wilton");
        self.pg_conn_config.enable_tls = true;
        self.pg_conn_config.accept_invalid_tls = true;

        self.set_status_bar_dbconn_label("none");
        self.open_connect_dialog(nwg::EventData::NoData);
    }

    pub(super) fn close(&mut self, _: nwg::EventData) {
        self.c.window.set_visible(false);
        nwg::stop_thread_dispatch();
    }

    pub(super) fn open_about_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = AboutDialogArgs::new(&self.c.about_notice);
        self.about_dialog_join_handle = AboutDialog::popup(args);
    }

    pub(super) fn await_about_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.about_notice.receive();
        let _ = self.about_dialog_join_handle.join();
        //self.c.filter_input.set_enabled(true);
    }

    pub(super) fn open_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(false);
        let args = ConnectDialogArgs::new(&self.c.connect_notice, self.pg_conn_config.clone());
        self.connect_dialog_join_handle = ConnectDialog::popup(args);
    }

    pub(super) fn await_connect_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.connect_notice.receive();
        let res = self.connect_dialog_join_handle.join();
        let mut dbnames: Vec<String> = res.dbnames.iter().filter(|name| {
            !vec!("msdb", "tempdb").contains(&name.as_str())
        }).map(|name| name.clone()).collect();
        dbnames.sort();
        self.c.backup_dbname_combo.set_collection(dbnames);
        self.c.backup_dbname_combo.set_selection(Some(1));
        self.on_dbname_changed(nwg::EventData::NoData);
        self.pg_conn_config = res.pg_conn_config;
        let sbar_label = format!(
            "{}:{}", &self.pg_conn_config.hostname, &self.pg_conn_config.port);
        self.set_status_bar_dbconn_label(&sbar_label);
    }

    pub(super) fn open_backup_command_dialog(&mut self, _: nwg::EventData) {
        let parent_dir = self.c.backup_dest_dir_input.text();
        let filename = self.c.backup_filename_input.text();
        let dirname = match Path::new(&filename).extension() {
            Some(ext) => filename.chars().take(filename.len() - (ext.len() + 1)).collect(),
            None => format!("{}_dir", filename)
        };
        let parent_dir_slashes = parent_dir.replace("\\", "/");
        let suffix = Uuid::new_v4().to_string().replace("-", "_");
        let dest_dir = format!("{}/{}_{}", parent_dir_slashes, dirname, suffix);
        println!("{}", dest_dir);
        // todo: bin path
        //let bin_dir = "C:\\Program Files\\WiltonDB Software\\wiltondb3.3\\bin";
        let bin_dir = "C:\\projects\\postgres\\dist\\bin";
        //let pg_dumpall = format!("{}\\pg_dumpall.exe", bin_dir);
        let pg_dump = format!("{}\\pg_dump.exe", bin_dir);
        // -h 127.0.0.1 -p 5432 -U wilton --bbf-database-name tmp1 -Fd -Z6 -f tmp1
        let pcc = &self.pg_conn_config;
        let dbname = match self.c.backup_dbname_combo.selection_string() {
            Some(name) => name,
            None => "todo".to_owned()
        };
        let cmd = PgCommand::new(pg_dump)
            .arg("-v")
            .arg("-h").arg(&pcc.hostname)
            .arg("-p").arg(&pcc.port.to_string())
            .arg("-U").arg(&pcc.username)
            .arg("--bbf-database-name").arg(&dbname)
            .arg("-Fd")
            .arg("-Z6")
            .arg("-f").arg(&dest_dir)
            .env_var("PGPASSWORD", &pcc.password)
            .zip_result_dir(&dest_dir, &filename)
            ;

        self.c.window.set_enabled(false);
        let args = CommandDialogArgs::new(&self.c.backup_command_notice, cmd);
        self.command_dialog_join_handle = CommandDialog::popup(args);
    }

    pub(super) fn await_backup_command_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.backup_command_notice.receive();
        let _ = self.command_dialog_join_handle.join();
    }

    pub(super) fn open_restore_command_dialog(&mut self, _: nwg::EventData) {
        let src_dir = self.c.restore_src_dir_input.text();
        // todo: bin path
        let bin_dir = "C:\\Program Files\\WiltonDB Software\\wiltondb3.3\\bin";
        let pg_restore = format!("{}\\pg_restore.exe", bin_dir);
        let pcc = &self.pg_conn_config;
        // todo:
        let dbname = self.c.restore_dbname_input.text();
        let cmd = PgCommand::new(pg_restore)
            .arg("-v")
            .arg("-h").arg(&pcc.hostname)
            .arg("-p").arg(&pcc.port.to_string())
            .arg("-U").arg(&pcc.username)
            .arg("-d").arg("wilton")
            .arg("-Fd")
            .arg(&src_dir)
            .env_var("PGPASSWORD", &pcc.password)
            .conn_config(pcc.clone())
            .sql(&format!("CREATE ROLE {}_db_owner", dbname))
            .sql(&format!("ALTER ROLE {}_db_owner WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", dbname))
            .sql(&format!("CREATE ROLE {}_dbo", dbname))
            .sql(&format!("ALTER ROLE {}_dbo WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", dbname))
            .sql(&format!("CREATE ROLE {}_guest", dbname))
            .sql(&format!("ALTER ROLE {}_guest WITH NOSUPERUSER INHERIT NOCREATEROLE NOCREATEDB NOLOGIN NOREPLICATION NOBYPASSRLS", dbname))
            .sql(&format!("GRANT {}_db_owner TO {}_dbo GRANTED BY sysadmin", dbname, dbname))
            .sql(&format!("GRANT {}_dbo TO sysadmin GRANTED BY sysadmin", dbname))
            .sql(&format!("GRANT {}_guest TO sysadmin GRANTED BY sysadmin", dbname))
            .sql(&format!("GRANT {}_guest TO {}_db_owner GRANTED BY sysadmin", dbname, dbname));
        self.c.window.set_enabled(false);
        let args = CommandDialogArgs::new(&self.c.backup_command_notice, cmd);
        self.command_dialog_join_handle = CommandDialog::popup(args);
    }

    pub(super) fn await_restore_command_dialog(&mut self, _: nwg::EventData) {
        self.c.window.set_enabled(true);
        self.c.restore_command_notice.receive();
        let _ = self.command_dialog_join_handle.join();
    }

    pub(super) fn open_website(&mut self, _: nwg::EventData) {
        let _ = Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("https://wiltondb.com")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    pub(super) fn on_resize(&mut self, _: nwg::EventData) {
        self.c.update_tab_order();
    }

    pub(super) fn choose_dest_dir(&mut self, _: nwg::EventData) {
        if let Ok(d) = std::env::current_dir() {
            if let Some(d) = d.to_str() {
                self.c.backup_dest_dir_chooser.set_default_folder(d).expect("Failed to set default folder.");
            }
        }

        if self.c.backup_dest_dir_chooser.run(Some(&self.c.window)) {
            self.c.backup_dest_dir_input.set_text("");
            if let Ok(directory) = self.c.backup_dest_dir_chooser.get_selected_item() {
                let dir = directory.into_string().unwrap();
                self.c.backup_dest_dir_input.set_text(&dir);
            }
        }
    }

    pub(super) fn choose_src_dir(&mut self, _: nwg::EventData) {
        if let Ok(d) = std::env::current_dir() {
            if let Some(d) = d.to_str() {
                self.c.restore_src_dir_chooser.set_default_folder(d).expect("Failed to set default folder.");
            }
        }

        if self.c.restore_src_dir_chooser.run(Some(&self.c.window)) {
            self.c.restore_src_dir_input.set_text("");
            if let Ok(directory) = self.c.restore_src_dir_chooser.get_selected_item() {
                let dir = directory.into_string().unwrap();
                self.c.restore_src_dir_input.set_text(&dir);
            }
        }
    }

    pub(super) fn on_dbname_changed(&mut self, _: nwg::EventData) {
        if let Some(name) = &self.c.backup_dbname_combo.selection_string() {
            let date = chrono::offset::Local::now();
            let date_st = date.format("%Y%m%d_%H%M%S");
            let filename = format!("{}_{}.zip", name, date_st);
            self.c.backup_filename_input.set_text(&filename);
        }
    }

    fn set_status_bar_dbconn_label(&self, text: &str) {
        self.c.status_bar.set_text(0, &format!("  DB connection: {}", text));
    }
}
