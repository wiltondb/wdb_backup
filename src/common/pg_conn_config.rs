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

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::time::Duration;

use native_tls::TlsConnector;
use postgres::Client;
use postgres::Config;
use postgres::NoTls;
use postgres_native_tls::MakeTlsConnector;

use super::*;

#[derive(Default, Debug, Clone)]
pub struct PgConnConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_pgpass_file: bool,
    pub connect_db: String,
    pub enable_tls: bool,
    pub accept_invalid_tls: bool,
}

impl PgConnConfig {
    pub fn open_connection_default(&self) -> Result<Client, PgAccessError> {
       self.open_connection(&self.connect_db)
    }

    pub fn open_connection_to_db(&self, dbname: &str) -> Result<Client, PgAccessError> {
        self.open_connection(dbname)
    }

    fn open_connection(&self, dbname: &str) -> Result<Client, PgAccessError> {
        let pwd = self.resolve_password()?;
        let conf = Config::new()
            .host(&self.hostname)
            .port(self.port)
            .user(&self.username)
            .password(&pwd)
            .dbname(dbname)
            .connect_timeout(Duration::from_secs(10))
            .clone();

        let res = if self.enable_tls {
            let connector = TlsConnector::builder()
                .danger_accept_invalid_certs(self.accept_invalid_tls)
                .danger_accept_invalid_hostnames(self.accept_invalid_tls)
                .build()?;
            let tls = MakeTlsConnector::new(connector);
            conf.connect(tls)?
        } else {
            conf.connect(NoTls)?
        };

        Ok(res)
    }

    fn resolve_password(&self) -> Result<String, PgAccessError> {
        if self.use_pgpass_file {
            let pgpass_path = PgConnConfig::resolve_pgpass_path()?;
            let file = File::open(&pgpass_path)?;
            let lines_it = BufReader::new(file).lines();
            let mut line_no = 1;
            for line_res in lines_it {
                let line = line_res?;
                let matched_opt = self.match_pgpass_line(&pgpass_path, &line, line_no)?;
                if let Some(pwd) = matched_opt {
                    return Ok(pwd)
                }
                line_no += 1;
            }
        }
        Ok(self.password.clone())
    }

    fn resolve_pgpass_path() -> Result<String, PgAccessError> {
        if let Ok(path_from_env) = std::env::var("PGPASSFILE") {
            Ok(path_from_env)
        } else if let Ok(appdir) = std::env::var("APPDATA") {
            Ok(format!("{}\\postgresql\\pgpass.conf", appdir))
        } else {
            Err(PgAccessError::pgpass_not_found())
        }
    }

    fn match_pgpass_line(&self, path: &str, line: &str, line_no: u32) -> Result<Option<String>, PgAccessError> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#") {
            return Ok(None);
        }
        let entry = PassfileEntry::new(path, trimmed.as_bytes(), line_no)?;
        if entry.hostname.accepts(&self.hostname)
            && entry.port.accepts(&self.port.to_string())
            && entry.dbname.accepts(&self.connect_db)
            && entry.user.accepts(&self.username)
        {
            return if entry.password.is_empty() {
                // To be consistent with libpq, in this case we need to stop searching the password file, but not
                // attempt to use the empty password string.
                Ok(None)
            } else {
                let password_st = String::from_utf8(entry.password)?;
                Ok(Some(password_st))
            }
        }
        Ok(None)
    }
}

// below impl is from https://github.com/sfackler/rust-postgres/pull/766

enum PassfileField {
    Wildcard,
    Bytes(Vec<u8>),
}

impl PassfileField {
    fn accepts(&self, value: &str) -> bool {
        match self {
            PassfileField::Wildcard => true,
            PassfileField::Bytes(b) => b == value.as_bytes(),
        }
    }
}

struct PassfileEntry {
    hostname: PassfileField,
    port: PassfileField,
    dbname: PassfileField,
    user: PassfileField,
    password: Vec<u8>,
}

impl PassfileEntry {
    fn new(path: &str, s: &[u8], line_no: u32) -> Result<PassfileEntry, PgAccessError> {
        let mut it = s.iter().copied();

        let mut parse_one_field = || {
            let mut value = Vec::new();
            let mut has_any_escape = false;
            while let Some(b) = it.next() {
                if b == b':' {
                    return Ok(match &value[..] {
                        b"*" if !has_any_escape => PassfileField::Wildcard,
                        _ => PassfileField::Bytes(value),
                    });
                } else if b == b'\\' {
                    has_any_escape = true;
                    value.push(it.next().ok_or(PgAccessError::pgpass_error(path, line_no))?);
                } else if b == b'\0' {
                    return Err(PgAccessError::pgpass_error(path, line_no));
                } else {
                    value.push(b)
                }
            }
            Err(PgAccessError::pgpass_error(path, line_no))
        };
        let hostname = parse_one_field()?;
        let port = parse_one_field()?;
        let dbname = parse_one_field()?;
        let user = parse_one_field()?;

        let mut parse_final_field = || {
            let mut value = Vec::new();
            while let Some(b) = it.next() {
                if b == b':' {
                    // Text that looks like an additional field is ignored
                    return Ok(value);
                } else if b == b'\\' {
                    // To be consistent with libpq, if the line ends with a backslash then the backslash is treated as
                    // part of the last field's value.
                    value.push(it.next().unwrap_or(b'\\'))
                } else if b == b'\0' {
                    return Err(PgAccessError::pgpass_error(path, line_no));
                } else {
                    value.push(b)
                }
            }
            Ok(value)
        };
        let password = parse_final_field()?;

        Ok(PassfileEntry {
            hostname,
            port,
            dbname,
            user,
            password,
        })
    }
}
