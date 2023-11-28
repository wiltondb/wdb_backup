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

mod args;
mod controls;
mod dialog;
mod events;
mod layout;
mod nui;
mod result;
mod toc;

use std::process;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use clipboard_win::formats;
use clipboard_win::set_clipboard;
use nwg::NativeUi;

use crate::*;
use common::PgConnConfig;
use common::unzip_directory;
use nwg_ui as ui;
use ui::Controls;
use ui::Events;
use ui::Layout;
use ui::PopupDialog;

pub use args::RestoreDialogArgs;
pub(self) use controls::RestoreDialogControls;
pub use dialog::RestoreDialog;
use events::RestoreDialogEvents;
use layout::RestoreDialogLayout;
pub use result::RestoreDialogResult;
use result::RestoreResult;
pub(self) use toc::rewrite_toc;