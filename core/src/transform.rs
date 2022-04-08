use std::borrow::Cow;

use hv_lua::{FromLua, ToLua};
use macroquad::prelude::*;
use tealr::{TypeBody, TypeName};

use crate::lua::wrapped_types::Vec2Lua;
#[derive(Debug, Default, tealr::TypeName, Clone)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32) -> Self {
        Transform { position, rotation }
    }
}

impl From<Vec2> for Transform {
    fn from(position: Vec2) -> Self {
        Transform {
            position,
            rotation: 0.0,
        }
    }
}
impl<'lua> ToLua<'lua> for Transform {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let transform = lua.create_table()?;
        let position = Vec2Lua::from(self.position).to_lua(lua)?;
        transform.set("position", position)?;
        transform.set("rotation", self.rotation)?;
        lua.pack(transform)
    }
}

impl<'lua> FromLua<'lua> for Transform {
    fn from_lua(
        value: hv_lua::Value<'lua>,
        _: &'lua hv_lua::Lua,
    ) -> std::result::Result<Self, hv_lua::Error> {
        let value = match value {
            hv_lua::Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "Table",
                    message: None,
                })
            }
        };
        let position = value.get::<_, Vec2Lua>("position")?.into();
        let rotation = value.get("rotation")?;

        Ok(Self { position, rotation })
    }
}
impl TypeBody for Transform {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("position").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("rotation").into(), f32::get_type_parts()));
    }
}
