use std::borrow::Cow;

use hv_lua::{FromLua, ToLua};
use tealr::{NameContainer, NamePart, TealType, TypeName};

#[derive(Debug, Clone, Copy)]
pub struct CopyComponent<T>(T);
impl<T: Send + Sync + 'static + Clone + Copy> hv_lua::UserData for CopyComponent<T>
where
    T: for<'a> ToLua<'a> + for<'a> FromLua<'a>,
{
    #[allow(clippy::unit_arg)]
    fn add_fields<'lua, F: hv_lua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| Ok(this.0));
        fields.add_field_method_set("value", |_, this, value| Ok(this.0 = value));
    }
    fn on_metatable_init(t: hv_alchemy::Type<Self>) {
        use hv_lua::hv::LuaUserDataTypeExt;
        t.add_clone().add_copy().mark_component();
    }
    fn add_type_methods<'lua, M: hv_lua::UserDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) {
        methods.add_function("new", |_, i: T| Ok(Self(i)));
    }
    fn on_type_metatable_init(t: hv_alchemy::Type<hv_alchemy::Type<Self>>) {
        use hv_lua::hv::LuaUserDataTypeTypeExt;
        t.mark_component_type();
    }
}
impl<T: tealr::TypeName> tealr::TypeName for CopyComponent<T> {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        let name = tealr::NamePart::Type(TealType {
            name: Cow::Borrowed("Component"),
            type_kind: tealr::KindOfType::External,
            generics: None,
        });
        let mut type_name = vec![name, NamePart::Symbol(Cow::Borrowed("<"))];
        type_name.append(&mut T::get_type_parts().into_owned());
        type_name.push(NamePart::Symbol(Cow::Borrowed(">")));
        Cow::Owned(type_name)
    }
}
impl<T: TypeName> tealr::TypeBody for CopyComponent<T> {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        get_body_component::<T, Self>(gen);
        //dbg!(gen);
    }
    fn get_type_body_marker(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        get_type_body_component::<T, Self>(gen);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CloneComponent<T>(T);
impl<T: Send + Sync + 'static + Clone> hv_lua::UserData for CloneComponent<T>
where
    T: for<'a> ToLua<'a> + for<'a> FromLua<'a>,
{
    #[allow(clippy::unit_arg)]
    fn add_fields<'lua, F: hv_lua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, this| Ok(this.0.clone()));
        fields.add_field_method_set("value", |_, this, value| Ok(this.0 = value));
    }
    fn on_metatable_init(t: hv_alchemy::Type<Self>) {
        use hv_lua::hv::LuaUserDataTypeExt;
        t.add_clone().mark_component();
    }
    fn add_type_methods<'lua, M: hv_lua::UserDataMethods<'lua, hv_alchemy::Type<Self>>>(
        methods: &mut M,
    ) {
        methods.add_function("new", |_, i: T| Ok(Self(i)));
    }
    fn on_type_metatable_init(t: hv_alchemy::Type<hv_alchemy::Type<Self>>) {
        use hv_lua::hv::LuaUserDataTypeTypeExt;
        t.mark_component_type();
    }
}
impl<T: tealr::TypeName> tealr::TypeName for CloneComponent<T> {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        let name = tealr::NamePart::Type(TealType {
            name: Cow::Borrowed("Component"),
            type_kind: tealr::KindOfType::External,
            generics: None,
        });
        let mut type_name = vec![name, NamePart::Symbol(Cow::Borrowed("<"))];
        type_name.append(&mut T::get_type_parts().into_owned());
        type_name.push(NamePart::Symbol(Cow::Borrowed(">")));
        Cow::Owned(type_name)
    }
}
impl<T: TypeName> tealr::TypeBody for CloneComponent<T> {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        get_body_component::<T, Self>(gen);
    }
    fn get_type_body_marker(gen: &mut tealr::TypeGenerator) {
        get_type_body_component::<T, Self>(gen);
    }
}

fn get_body_component<T: TypeName, SelfType: TypeName>(gen: &mut tealr::TypeGenerator) {
    gen.fields.push((
        "value".as_bytes().to_vec().into(),
        <T as tealr::TypeName>::get_type_parts(),
    ));
}

fn get_type_body_component<T: TypeName, SelfType: TypeName>(gen: &mut tealr::TypeGenerator) {
    let mut signature = vec![NamePart::Symbol(Cow::Borrowed("function("))];
    signature.append(&mut T::get_type_parts().into_owned());
    signature.push(NamePart::Symbol(Cow::Borrowed("):")));
    signature.append(&mut SelfType::get_type_parts().into_owned());
    gen.methods.push(tealr::ExportedFunction {
        name: Cow::Borrowed("new").into(),
        signature: Cow::Owned(signature),
        is_meta_method: false,
    });
}
#[macro_export]
macro_rules! create_type_component_container {
    ($name:ident with $($field_name:ident of $type_name:ty,)+) => {
        #[derive(Debug, Clone)]
        pub struct $name<'lua>(hv_lua::Table<'lua>);
        impl<'lua> hv_lua::ToLua<'lua> for $name<'lua> {
            fn to_lua(
                self,
                lua: &'lua hv_lua::Lua,
            ) -> std::result::Result<hv_lua::Value<'lua>, hv_lua::Error> {
                self.0.to_lua(lua)
            }
        }
        impl<'lua> tealr::TypeName for $name<'lua> {
            fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
                use tealr::new_type;
                new_type!(Components)
            }
        }
        impl<'lua> tealr::TypeBody for $name<'lua> {
            fn get_type_body(gen: &mut tealr::TypeGenerator) {
                $(
                    println!("pushing field: {}",stringify!($type_name));
                    gen.fields.push(
                        (

                            stringify!($field_name).as_bytes().to_vec().into(),
                            <$type_name as tealr::TypeName>::get_marker_type_parts()

                        )
                    );
                )*

            }
        }
        impl<'lua> $name<'lua> {
            pub fn new(lua: &'lua hv_lua::Lua) -> Result<Self, Box<dyn std::error::Error>> {
                let table = lua.create_table()?;
                $(
                    table.set(stringify!($field_name),lua.create_userdata_type::<$type_name>()?)?;
                )*
                Ok(Self(table))
            }
        }
    };
}
