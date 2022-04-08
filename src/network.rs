//! This module holds the networking core, used

use std::sync::Arc;

use hecs::World;
use hv_cell::AtomicRefCell;

pub fn update_network_client(world: Arc<AtomicRefCell<World>>) {
    update_network_common(world);
}

pub fn fixed_update_network_client(world: Arc<AtomicRefCell<World>>) {
    fixed_update_network_common(world);
}

pub fn update_network_host(world: Arc<AtomicRefCell<World>>) {
    update_network_common(world);
}

pub fn fixed_update_network_host(world: Arc<AtomicRefCell<World>>) {
    fixed_update_network_common(world);
}

fn update_network_common(_world: Arc<AtomicRefCell<World>>) {}

fn fixed_update_network_common(_world: Arc<AtomicRefCell<World>>) {}
