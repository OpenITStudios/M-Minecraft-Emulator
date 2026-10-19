// Only the active native pilot belongs to the current desktop compile graph.
// Legacy VMware and screen-overlay sources remain in Git but are unmounted.
pub mod app_instance;
pub mod dpi;
pub mod instance_catalog;
pub mod instance_discovery;
pub mod instance_focus;
pub mod instance_focus_events;
pub mod instance_identity;
pub mod native_clients;
pub mod owned_isolation;
pub mod virtual_inventory;
