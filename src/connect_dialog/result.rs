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
pub struct ConnectDialogResult {
    pub pg_conn_config: PgConnConfig,
    pub load_settings_requested: bool,
}

impl ConnectDialogResult {
    pub fn new(pg_conn_config: PgConnConfig) -> Self {
        Self {
            pg_conn_config,
            load_settings_requested: true
        }
    }

    pub fn cancelled() -> Self {
        Self {
            pg_conn_config: Default::default(),
            load_settings_requested: false
        }
    }
}
