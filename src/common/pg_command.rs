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

use std::collections::HashMap;
use std::path::Path;
use crate::common::PgConnConfig;

#[derive(Default, Debug, Clone)]
pub struct PgCommandZip {
    pub enabled: bool,
    pub dir_path: String,
    pub zip_file_path: String,
    pub comp_level: u8,
}

impl PgCommandZip {
    fn new(dir: &str, zip_file: &str) -> Self {
        let dir_path = Path::new(dir);
        // todo: fixme
        let parent_path = dir_path.parent().expect("Parent path fail");
        Self {
            enabled: true,
            dir_path: dir_path.to_string_lossy().to_string(),
            zip_file_path: parent_path.join(Path::new(zip_file)).to_string_lossy().to_string(),
            comp_level: 0
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PgCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub sql_statements: Vec<String>,
    pub conn_config: PgConnConfig,
    pub zip_result_dir: PgCommandZip,
}

impl PgCommand {
    pub fn new(program: String) -> Self {
       Self {
           program,
           ..Default::default()
       }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn env_var(mut self, name: &str, value: &str) -> Self {
        self.env_vars.insert(name.to_string(), value.to_string());
        self
    }

    pub fn sql(mut self, sql: &str) -> Self {
        self.sql_statements.push(sql.to_string());
        self
    }

    pub fn conn_config(mut self, conn_config: PgConnConfig) -> Self {
        self.conn_config = conn_config;
        self
    }

    pub fn zip_result_dir(mut self, result_dir: &str, zip_file_name: &str) -> Self {
        self.zip_result_dir = PgCommandZip::new(result_dir, zip_file_name);
        self
    }
}