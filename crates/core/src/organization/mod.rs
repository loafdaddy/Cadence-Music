//! Optional, non-destructive library organization.
//!
//! Cadence never touches the user's files unless organization is explicitly
//! enabled. Even then, the flow is always:
//!
//! 1. Build a [`plan::OrganizationPlan`] describing every proposed move.
//! 2. Show the plan to the user for review (nothing has happened on disk yet).
//! 3. On confirmation, [`plan::OrganizationPlan::execute`] performs the moves
//!    and returns an [`plan::UndoLog`] that can reverse them.
//!
//! Filenames are always sanitized to be valid on common filesystems, and
//! collisions are detected up front rather than clobbering existing files.

mod plan;
mod template;

pub use plan::{FileMove, OrganizationPlan, PlanEntry, UndoLog};
pub use template::{sanitize_component, Preset, Template};
