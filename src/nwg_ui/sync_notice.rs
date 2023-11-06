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

use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enablewindow
// A window must be enabled before it can be activated. For example, if an application
// is displaying a modeless dialog box and has disabled its main window, the application
// must enable the main window before destroying the dialog box. Otherwise, another window
// will receive the keyboard focus and be activated.

pub type SyncNotice = SyncNoticeValue<()>;
pub type SyncNoticeSender = SyncNoticeValueSender<()>;

#[derive(Default)]
pub struct SyncNoticeValueSender<T: Sync + Send + Default> {
    sender: Option<nwg::NoticeSender>,
    tx: Option<SyncSender<T>>,
}

impl<T: Sync + Send + Default> SyncNoticeValueSender<T> {
    pub fn send(&self) {
        self.send_result(Default::default());
    }

    pub fn send_result(&self, t: T) {
        self.sender.as_ref().expect("Sender not initialized").notice();
        // best effort, receiver may have been destroyed already
        let _ = self.tx.as_ref().expect("Sender not initialized").send(t);
    }
}

#[derive(Default)]
pub struct SyncNoticeValue<T: Sync + Send + Default> {
    pub notice: nwg::Notice,
    tx: Option<SyncSender<T>>,
    rx: Option<Receiver<T>>,
    receive_count: usize,
}

#[allow(dead_code)]
impl<T: Sync + Send + Default> SyncNoticeValue<T> {

    pub fn sender(&self) -> SyncNoticeValueSender<T> {
        SyncNoticeValueSender {
            sender: Some(self.notice.sender()),
            tx: Some(self.tx.as_ref().expect("Notice not initialized").clone()),
        }
    }

    pub fn receive(&mut self) -> T {
        let res = self.rx.as_ref().expect("Notice not initialized")
            .recv().expect("Notice receive failure");
        self.receive_count += 1;
        res
    }

    pub fn receive_count(&self) -> usize {
        self.receive_count
    }
}

pub struct SyncNoticeBuilder {
    parent: Option<nwg::ControlHandle>,
}

impl SyncNoticeBuilder {
    pub(super) fn new() -> Self {
        Self {
            parent: None,
        }
    }

    pub fn parent<C: Into<nwg::ControlHandle>>(mut self, p: C) -> Self {
        self.parent = Some(p.into());
        self
    }

    pub fn build<T: Sync + Send + Default>(self, out: &mut SyncNoticeValue<T>) -> Result<(), nwg::NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(nwg::NwgError::no_parent("Notice"))
        }?;

        nwg::Notice::builder()
            .parent(&parent)
            .build(&mut out.notice)?;

        let (tx, rx) = sync_channel::<T>(0);
        out.tx = Some(tx);
        out.rx = Some(rx);

        out.receive_count = 0;

        Ok(())
    }

}
