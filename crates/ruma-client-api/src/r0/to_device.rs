//! Endpoints for client devices to exchange information not persisted in room DAG.

pub use ruma_common::to_device::DeviceIdOrAllDevices;

pub mod send_event_to_device;
