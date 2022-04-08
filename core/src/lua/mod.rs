mod create_component;
pub mod wrapped_types;
pub use create_component::{CloneComponent, CopyComponent};

use std::{error::Error, os::unix::prelude::OsStrExt, path::Path};

use hv_lua::{chunk, hv::types, Function, Lua, Table, Value};

const LUA_ENTRY: &str = "main";

crate::create_type_component_container!(
    TypeContainer with
    I32 of CopyComponent<i32>,
    Bool of CopyComponent<bool>,
);

pub fn init_lua<RegisterTypes: Fn(&Lua) -> Result<Value, Box<dyn Error>>>(
    mod_dir: &Path,
    register_types: RegisterTypes,
) -> Result<Lua, Box<dyn Error>> {
    let lua = Lua::new();
    let hv = types(&lua)?;
    let component_types = register_types(&lua)?;
    let dir = mod_dir.as_os_str().as_bytes().to_owned();
    {
        let globals = lua.globals();
        //a table containing all the mods
        globals.set("mods", lua.create_table()?)?;
        //this table is used to have quick access to every event that needs to run.
        globals.set("events", lua.create_table()?)?;

        //we don't want users to simply use `require` to load their files as that will either result in conflicts or annoying path names
        //to drive this point home, I rename the `require` function here and add 2 other functions to load mod files
        let req = globals.get::<_, Function>("require")?;
        globals.set("require_lib", req)?;
        globals.set("require", hv_lua::Nil)?;

        globals.set("hv", hv)?;
        globals.set("type_components", component_types)?;

        //this function allows people to load a file from arbitrary mods
        //this allows "library only" mods to exist.
        //there will also be a function that loads from the current mod. However, that one needs to be defined when loading the mod
        let load_any_mod_file = lua.create_function(
            move |lua, (from_mod, path): (hv_lua::String, hv_lua::String)| {
                //TODO: a way to go from `mod id` to `mod folder`
                let globals = lua.globals();
                let req = globals.get::<_, Function>("require_lib")?;
                let mod_folder = globals
                    .get::<_, Table>("mods")?
                    .get::<_, Table>(from_mod)?
                    .get::<_, hv_lua::String>("dir_name")?;

                let mut full_path = Vec::new();
                full_path.extend_from_slice(&dir);
                full_path.extend_from_slice(b".");
                full_path.extend_from_slice(mod_folder.as_bytes());
                full_path.extend_from_slice(b".");
                full_path.extend_from_slice(path.as_bytes());
                let full_path = lua.create_string(&full_path)?;
                req.call::<_, hv_lua::Value>(full_path)
            },
        )?;
        globals.set("require_from", load_any_mod_file)?;
    }
    Ok(lua)
}

pub fn load_lua<P: AsRef<[u8]>>(
    mod_id: String,
    mod_folder: P,
    lua: &Lua,
) -> Result<(), Box<dyn Error>> {
    let name = mod_id;
    let name_to_transfer = name.clone();
    //ideally, we use a lua string but... those don't get transferred properly for some reason....
    let dir = String::from_utf8_lossy(mod_folder.as_ref());
    let entry = LUA_ENTRY.to_string();
    let chunk = chunk! {
        local name = $name_to_transfer
        local dir = $dir
        local entry = $entry

        function require(load_path)
            return require_from(name , load_path)
        end
        local function init_mod()
            return require(entry)
        end
        local mod_config = {
            dir_name = dir,
            require = require,
            mod_id = name
        }
        mods[name] = mod_config

        local mod_events = init_mod()
        mod_config["events"] = mod_events
        if type(mod_events) == "table" then
            for k, _ in pairs(mod_events) do
                local event_list = events[k] or {}
                table.insert(event_list, mod_config)
                events[k] = event_list
            end
        end
        require = nil
    };
    lua.load(chunk)
        .set_name(&format!("Load mod: {:?}", name))?
        .exec()?;

    Ok(())
}

pub fn get_table(value: hv_lua::Value) -> Result<Table, hv_lua::Error> {
    match value {
        hv_lua::Value::Table(x) => Ok(x),
        v => Err(hv_lua::Error::FromLuaConversionError {
            from: v.type_name(),
            to: "table",
            message: None,
        }),
    }
}
