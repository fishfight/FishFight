use macroquad::math::Vec2;
use macroquad::prelude::{collections::storage, get_frame_time};
use macroquad_platformer::Tile;

use crate::GameWorld;

use crate::components::PhysicsBody;

//at the moment is not possible to test the code as ite requires EruptingVolcano to test
//It also is dependency of ArmedGrenade so a rough estimation of the code to not implement the trait later
//is here.
pub trait EruptedItem {
    fn spawn_for_volcano(pos: Vec2, speed: Vec2, enable_at_y: f32, owner_id: u8);

    fn body(&mut self) -> &mut PhysicsBody;
    fn enable_at_y(&self) -> f32;

    // Assumes that the eruption is running; doesn't check it.
    fn eruption_update(&mut self) -> bool {
        let enable_at_y = self.enable_at_y();
        let body = self.body();

        // Controls the Actor as long as is erupting,
        // afterwards it informs the actor update to stop calling this function

        body.position.y += PhysicsBody::GRAVITY * get_frame_time().powi(2) / 2.
            + body.velocity.y * get_frame_time();
        body.position.x += body.velocity.x * get_frame_time();
        body.velocity.y += PhysicsBody::GRAVITY * get_frame_time();

        if body.position.y < enable_at_y || body.velocity.y < 0. {
            return false;
        }

        let collision_world = &mut storage::get_mut::<GameWorld>().collision_world;

        let tile = collision_world.collide_solids(body.position, 15, 15);

        if tile != Tile::Empty {
            return false;
        }

        body.collider = collision_world.add_actor(body.position, 15, 15);

        true
    }
}
