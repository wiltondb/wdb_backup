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
use crate::common::PgConnConfig;

#[derive(Default, Debug, Clone)]
pub struct PgCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub sql_statements: Vec<String>,
    pub conn_config: PgConnConfig,
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
}