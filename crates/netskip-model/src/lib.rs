// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Net Skip's browser model: the rules a browser follows that do not depend on any UI.
//!
//! - [`address`]: what the address bar's text means, and how an address is shown.
//! - [`search`]: the search engines, their query and suggestion URLs.
//! - [`complete`]: inline completion and the suggestion list.
//! - [`history`]: what is recorded, day grouping, clearing.
//! - [`tabs`]: new-tab placement, recently closed tabs, the saved session.
//! - [`fixtures`]: the address mapping that serves test pages without a network.

pub mod address;
pub mod complete;
pub mod fixtures;
pub mod history;
pub mod search;
pub mod tabs;
