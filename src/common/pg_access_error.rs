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

use std::fmt;
use std::io;

#[derive(Debug)]
pub struct PgAccessError {
    message: String
}

impl PgAccessError {
    pub fn new<E: fmt::Display>(e: &E) -> Self {
        Self {
            message: format!("{}", e)
        }
    }

    pub fn from_string(message: String) -> Self {
        Self {
            message
        }
    }

    pub fn pgpass_not_found() -> Self {
        Self {
            message: "pgpass file not found on path: '%APPDATA%/postgresql/pgpass.conf'".to_string()
        }
    }

    pub fn pgpass_error(path: &str, line_no: u32) -> Self {
        Self {
            message: format!(
                "Error reading password from pgpass file on path: [{}], line number: [{}]", path, line_no)
        }
    }
}

impl fmt::Display for PgAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<postgres::Error> for PgAccessError {
    fn from(value: postgres::Error) -> Self {
        Self::new(&value)
    }
}

impl From<native_tls::Error> for PgAccessError {
    fn from(value: native_tls::Error) -> Self {
        Self::new(&value)
    }
}

impl From<io::Error> for PgAccessError {
    fn from(value: io::Error) -> Self {
        Self::new(&value)
    }
}

impl From<&str> for PgAccessError {
    fn from(value: &str) -> Self {
        Self::new(&value)
    }
}

impl From<std::string::FromUtf8Error> for PgAccessError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::new(&value)
    }
}
