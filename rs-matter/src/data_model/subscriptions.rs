/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use core::cell::RefCell;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Instant;

use portable_atomic::{AtomicU32, Ordering};

use crate::utils::notification::Notification;

struct Subscription {
    node_id: u64,
    id: u32,
    // We use u16 instead of embassy::Duration to save some storage
    min_int_secs: u16,
    // Ditto
    max_int_secs: u16,
    reported_at: Instant,
    changed: bool,
}

impl Subscription {
    pub fn report_due(&self, now: Instant) -> bool {
        self.changed
            && self.reported_at + embassy_time::Duration::from_secs(self.min_int_secs as _) <= now
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        self.reported_at + embassy_time::Duration::from_secs(self.max_int_secs as _) <= now
    }
}

/// A utility for tracking subscriptions accepted by the data model.
///
/// The `N` type parameter specifies the maximum number of subscriptions that can be tracked at the same time.
/// Additional subscriptions are rejected by the data model with a "resource exhausted" IM status message.
pub struct Subscriptions<const N: usize> {
    next_subscription_id: AtomicU32,
    subscriptions: RefCell<heapless::Vec<Subscription, N>>,
    pub(crate) notification: Notification<NoopRawMutex>,
}

impl<const N: usize> Subscriptions<N> {
    /// Create the instance.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            next_subscription_id: AtomicU32::new(1),
            subscriptions: RefCell::new(heapless::Vec::new()),
            notification: Notification::new(),
        }
    }

    /// Notify the instance that some data in the data model has changed and that it should re-evaluate the subscriptions
    /// and report on those that concern the changed data.
    ///
    /// This method is supposed to be called by the application code whenever it changes the data model.
    pub fn notify_changed(&self) {
        for sub in self.subscriptions.borrow_mut().iter_mut() {
            sub.changed = true;
        }

        self.notification.notify();
    }

    pub(crate) fn add(&self, node_id: u64, min_int_secs: u16, max_int_secs: u16) -> Option<u32> {
        let id = self.next_subscription_id.fetch_add(1, Ordering::SeqCst);

        self.subscriptions
            .borrow_mut()
            .push(Subscription {
                node_id,
                id,
                min_int_secs,
                max_int_secs,
                reported_at: Instant::MAX,
                changed: false,
            })
            .map(|_| id)
            .ok()
    }

    /// Mark the subscription with the given ID as reported.
    ///
    /// Will return `false` if the subscription with the given ID does no longer exist, as it might be
    /// removed by a concurrent transaction while being reported on.
    pub(crate) fn mark_reported(&self, id: u32) -> bool {
        let mut subscriptions = self.subscriptions.borrow_mut();

        if let Some(sub) = subscriptions.iter_mut().find(|sub| sub.id == id) {
            sub.reported_at = Instant::now();
            sub.changed = false;

            true
        } else {
            false
        }
    }

    pub(crate) fn remove(&self, node_id: Option<u64>, id: Option<u32>) {
        let mut subscriptions = self.subscriptions.borrow_mut();
        while let Some(index) = subscriptions.iter().position(|sub| {
            sub.node_id == node_id.unwrap_or(sub.node_id) && sub.id == id.unwrap_or(sub.id)
        }) {
            subscriptions.swap_remove(index);
        }
    }

    pub(crate) fn find_expired(&self, now: Instant) -> Option<(u64, u32)> {
        self.subscriptions
            .borrow()
            .iter()
            .find_map(|sub| sub.is_expired(now).then_some((sub.node_id, sub.id)))
    }

    /// Note that this method has a side effect:
    /// it updates the `reported_at` field of the subscription that is returned.
    pub(crate) fn find_report_due(&self, now: Instant) -> Option<(u64, u32)> {
        self.subscriptions
            .borrow_mut()
            .iter_mut()
            .find(|sub| sub.report_due(now))
            .map(|sub| {
                sub.reported_at = now;
                (sub.node_id, sub.id)
            })
    }
}
