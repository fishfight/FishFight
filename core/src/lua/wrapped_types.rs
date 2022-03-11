use std::borrow::Cow;

use hv_lua::{FromLua, ToLua, UserData, Value};
use macroquad::prelude::{Color, Rect, Texture2D, Vec2};
use tealr::{
    mlu::{TealData, UserDataWrapper},
    new_type, TypeBody, TypeName,
};

#[derive(Clone)]
pub struct Vec2Lua {
    pub x: f32,
    pub y: f32,
}
impl From<Vec2> for Vec2Lua {
    fn from(x: Vec2) -> Self {
        Self { x: x.x, y: x.y }
    }
}
impl From<Vec2Lua> for Vec2 {
    fn from(x: Vec2Lua) -> Self {
        Self::new(x.x, x.y)
    }
}
impl<'lua> ToLua<'lua> for Vec2Lua {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let position = lua.create_table()?;
        position.set("x", self.x)?;
        position.set("y", self.y)?;
        lua.pack(position)
    }
}

impl<'lua> FromLua<'lua> for Vec2Lua {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let value = match lua_value {
            hv_lua::Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "Table",
                    message: None,
                })
            }
        };
        let x = value.get("x")?;
        let y = value.get("y")?;
        Ok(Self { x, y })
    }
}

impl TypeName for Vec2Lua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Vec, External)
    }
}
impl TypeBody for Vec2Lua {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("x"), Cow::Borrowed("number")));
        gen.fields
            .push((Cow::Borrowed("y"), Cow::Borrowed("number")));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorLua {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl TypeName for ColorLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Color, External)
    }
}
impl From<Color> for ColorLua {
    fn from(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

impl From<ColorLua> for Color {
    fn from(color: ColorLua) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}
impl<'lua> FromLua<'lua> for ColorLua {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let v = match lua_value {
            Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "table",
                    message: None,
                })
            }
        };
        Ok(Self {
            r: v.get("r")?,
            g: v.get("g")?,
            b: v.get("b")?,
            a: v.get("a")?,
        })
    }
}
impl<'lua> ToLua<'lua> for ColorLua {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Value<'lua>> {
        let table = lua.create_table()?;
        table.set("r", self.r)?;
        table.set("g", self.g)?;
        table.set("b", self.b)?;
        table.set("a", self.a)?;
        lua.pack(table)
    }
}

#[derive(Clone)]
pub struct RectLua(Rect);
impl From<RectLua> for Rect {
    fn from(r: RectLua) -> Self {
        r.0
    }
}
impl From<Rect> for RectLua {
    fn from(x: Rect) -> Self {
        Self(x)
    }
}
impl TypeName for RectLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Rect, External)
    }
}
impl UserData for RectLua {
    fn add_fields<'lua, F: hv_lua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |lua, this| this.0.x.to_lua(lua));
        fields.add_field_method_get("y", |lua, this| this.0.y.to_lua(lua));
        fields.add_field_method_get("w", |lua, this| this.0.w.to_lua(lua));
        fields.add_field_method_get("h", |lua, this| this.0.h.to_lua(lua));
        fields.add_field_method_set("x", |_, this, value| {
            this.0.x = value;
            Ok(())
        });
        fields.add_field_method_set("y", |_, this, value| {
            this.0.y = value;
            Ok(())
        });
        fields.add_field_method_set("w", |_, this, value| {
            this.0.w = value;
            Ok(())
        });
        fields.add_field_method_set("h", |_, this, value| {
            this.0.h = value;
            Ok(())
        });
    }

    fn add_methods<'lua, M: hv_lua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
}
impl TealData for RectLua {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("point", |lua, this, ()| {
            Vec2Lua::from(this.0.point()).to_lua(lua)
        });
        methods.add_method("size", |lua, this, ()| {
            Vec2Lua::from(this.0.size()).to_lua(lua)
        });
        methods.add_method("left", |lua, this, ()| this.0.left().to_lua(lua));
        methods.add_method("right", |lua, this, ()| this.0.right().to_lua(lua));
        methods.add_method("top", |lua, this, ()| this.0.top().to_lua(lua));
        methods.add_method("bottom", |lua, this, ()| this.0.bottom().to_lua(lua));
        methods.add_method_mut("move_to", |_, this, vec: Vec2Lua| {
            this.0.move_to(vec.into());
            Ok(())
        });
        methods.add_method_mut("scale", |_, this, (sx, sy)| {
            this.0.scale(sx, sy);
            Ok(())
        });
        methods.add_method("contains", |lua, this, point: Vec2Lua| {
            this.0.contains(point.into()).to_lua(lua)
        });
        methods.add_method("overlaps", |lua, this, other: RectLua| {
            this.0.overlaps(&other.into()).to_lua(lua)
        });
        methods.add_method("combine_with", |lua, this, other: RectLua| {
            RectLua::from(this.0.combine_with(other.into())).to_lua(lua)
        });
        methods.add_method("intersect", |lua, this, other: RectLua| {
            this.0
                .intersect(other.into())
                .map(RectLua::from)
                .to_lua(lua)
        });
        methods.add_method("offset", |lua, this, offset: Vec2Lua| {
            RectLua::from(this.0.offset(offset.into())).to_lua(lua)
        });
    }
}
impl TypeBody for RectLua {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_methods(gen);
        gen.fields.push((
            Cow::Borrowed("x"),
            tealr::type_parts_to_str(f32::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("y"),
            tealr::type_parts_to_str(f32::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("w"),
            tealr::type_parts_to_str(f32::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("h"),
            tealr::type_parts_to_str(f32::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("x"),
            tealr::type_parts_to_str(f32::get_type_parts()),
        ));
    }
}

#[derive(Clone)]
pub struct Texture2DLua(Texture2D);
impl TypeName for Texture2DLua {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        new_type!(Texture2D, External)
    }
}
impl UserData for Texture2DLua {}
impl TypeBody for Texture2DLua {
    fn get_type_body(_: &mut tealr::TypeGenerator) {}
}

impl From<Texture2DLua> for Texture2D {
    fn from(x: Texture2DLua) -> Self {
        x.0
    }
}
impl From<Texture2D> for Texture2DLua {
    fn from(x: Texture2D) -> Self {
        Self(x)
    }
}
