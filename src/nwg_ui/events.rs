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

pub trait Events<C: Controls> {
    fn build(&mut self, c: &C) -> Result<(), nwg::NwgError>;
}

pub struct Event<W> {
    pub control_handle: nwg::ControlHandle,
    pub event: nwg::Event,
    pub handler: fn(&mut W, nwg::EventData) -> ()
}

pub struct EventBuilder<W> {
    control_handle: Option<nwg::ControlHandle>,
    event: Option<nwg::Event>,
    handler: Option<fn(&mut W, nwg::EventData) -> ()>
}

impl<W> EventBuilder<W> {
    pub(super) fn new() -> Self {
        Self {
            control_handle: None,
            event: None,
            handler: None
        }
    }

    pub fn control<C: Into<nwg::ControlHandle>>(mut self, control: C) -> Self {
        self.control_handle = Some(control.into());
        self
    }

    pub fn event(mut self, event: nwg::Event) -> Self {
        self.event = Some(event);
        self
    }

    pub fn handler(mut self, handler: fn(&mut W, nwg::EventData) -> ()) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self, events: &mut Vec<Event<W>>) -> Result<(), nwg::NwgError> {
        let control_handle = match self.control_handle {
            None => return Err(nwg::NwgError::events_binding("Control not specified".to_string())),
            Some(ch) => Ok::<nwg::ControlHandle, nwg::NwgError>(ch)
        }?;
        let event = match self.event {
            None => return Err(nwg::NwgError::events_binding("Event not specified".to_string())),
            Some(ev) => Ok::<nwg::Event, nwg::NwgError>(ev)
        }?;
        let handler = match self.handler {
            None => return Err(nwg::NwgError::events_binding("Handler not specified".to_string())),
            Some(h) => Ok::<fn(&mut W, nwg::EventData) -> (), nwg::NwgError>(h)
        }?;

        events.push(Event {
            control_handle, event, handler
        });

        Ok(())
    }
}
