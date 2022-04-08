use std::{error::Error, sync::Arc};

use hecs::{Entity, World};
use hv_alchemy::Type;
use hv_cell::AtomicRefCell;
use tealr::mlu::mlua::{
    chunk,
    hv::{types, LuaUserDataTypeExt, LuaUserDataTypeTypeExt},
    Lua, UserData, UserDataFields, UserDataMethods,
};

#[derive(Debug, Clone, Copy)]
pub struct I32Component(i32);
impl UserData for I32Component {
    #[allow(clippy::unit_arg)]
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| Ok(this.0));
        fields.add_field_method_set("value", |_, this, value| Ok(this.0 = value));
    }
    fn on_metatable_init(t: Type<Self>) {
        t.add_clone().add_copy().mark_component();
    }

    // The following methods are a bit like implementing `UserData` on `Type<Self>`, the userdata
    // type object of `Self`. This one just lets you construct an `I32Component` from Lua given a
    // value convertible to an `i32`.
    fn add_type_methods<'lua, M: UserDataMethods<'lua, Type<Self>>>(methods: &mut M) {
        methods.add_function("new", |_, i: i32| Ok(Self(i)));
    }

    // We want to generate the necessary vtables for accessing this type as a component in the ECS.
    // The `LuaUserDataTypeTypeExt` extension trait provides convenient methods for registering the
    // required traits for this (`.mark_component_type()` is shorthand for
    // `.add::<dyn ComponentType>()`.)
    fn on_type_metatable_init(t: Type<Type<Self>>) {
        t.mark_component_type();
    }
}
#[derive(Debug, Clone, Copy)]
pub struct BoolComponent(bool);
impl UserData for BoolComponent {
    #[allow(clippy::unit_arg)]
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| Ok(this.0));
        fields.add_field_method_set("value", |_, this, value| Ok(this.0 = value));
    }
    fn on_metatable_init(t: Type<Self>) {
        t.add_clone().add_copy().mark_component();
    }

    // The following methods are a bit like implementing `UserData` on `Type<Self>`, the userdata
    // type object of `Self`. This one just lets you construct an `BoolComponent` from Lua given a
    // value convertible to an `i32`.
    fn add_type_methods<'lua, M: UserDataMethods<'lua, Type<Self>>>(methods: &mut M) {
        methods.add_function("new", |_, i: bool| Ok(Self(i)));
    }

    // We want to generate the necessary vtables for accessing this type as a component in the ECS.
    // The `LuaUserDataTypeTypeExt` extension trait provides convenient methods for registering the
    // required traits for this (`.mark_component_type()` is shorthand for
    // `.add::<dyn ComponentType>()`.)
    fn on_type_metatable_init(t: Type<Type<Self>>) {
        t.mark_component_type();
    }
}

pub fn test() -> Result<(), Box<dyn Error>> {
    let lua = Lua::new();
    let hv = types(&lua)?;
    let i32_ty = lua.create_userdata_type::<I32Component>()?;
    let bool_ty = lua.create_userdata_type::<BoolComponent>()?;

    let world = Arc::new(AtomicRefCell::new(World::new()));
    // Clone the world so that it doesn't become owned by Lua. We still want a copy!
    let world_clone = world.clone();

    let chunk = chunk! {
        print("Hello from lua!")
        // Drag in the `hv` table we created above, and also the `I32Component` and `BoolComponent` types,
        // presumptuously calling them `I32` and `Bool` just because they're wrappers around the fact we
        // can't just slap a primitive in there and call it a day.
        local hv = $hv
        local Query = hv.ecs.Query
        local I32, Bool = $i32_ty, $bool_ty

        local world = $world_clone
        // Spawn an entity, dynamically adding components to it taken from userdata! Works with copy,
        // clone, *and* non-clone types (non-clone types will be moved out of the userdata and the userdata
        // object marked as destructed)
        print("make new entity with I32(5) and Bool(true)")
        local entity = world:spawn { I32.new(5), Bool.new(true) }
        print("Query it")
        // Dynamic query functionality, using our fork's `hecs::DynamicQuery`.
        local query = Query.new { Query.write(I32), Query.read(Bool) }
        // Querying takes a closure in order to enforce scope - the queryitem will panic if used outside that
        // scope.
        world:query_one(query, entity, function(item)
            print("Got item:",item)
            // Querying allows us to access components of our item as userdata objects through the same interface
            // we defined above!
            print("asserting if still true")
            assert(item:take(Bool).value == true)
            local i = item:take(I32)
            print("asserting if still 5")
            assert(i.value == 5)
            print("time to set it to 6")
            i.value = 6
            assert(i.value == 6)
        end)
        print("Returning it from lua, back to rust")
        // Return the entity we spawned back to Rust so we can examine it there.
        return entity
    };
    let entity: Entity = lua.load(chunk).eval()?;

    // Look! It worked!
    let borrowed = world.borrow();
    println!("Querying from rust and asserting_eq");
    let mut q = borrowed
        .query_one::<(&I32Component, &BoolComponent)>(entity)
        .ok();
    assert_eq!(
        q.as_mut().and_then(|q| q.get()).map(|(i, b)| (i.0, b.0)),
        Some((6, true))
    );
    Ok(())
}
