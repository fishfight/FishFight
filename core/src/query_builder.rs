use std::{
    fmt::Display,
    marker::PhantomData,
    sync::{Arc, Mutex, PoisonError},
};

use hecs::{Entity, Query, QueryBorrow, QueryMut, World};

#[derive(Clone)]
pub struct BuildStart();
pub struct Builder1<T>(PhantomData<T>);
impl<T> Clone for Builder1<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
pub struct Builder2<A, B>(PhantomData<(A, B)>);
impl<A, B> Clone for Builder2<A, B> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
pub struct Builder3<A, B, C>(PhantomData<(A, B, C)>);
impl<A, B, C> Clone for Builder3<A, B, C> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
pub struct Builder4<A, B, C, D>(PhantomData<(A, B, C, D)>);
pub struct Builder5<A, B, C, D, E>(PhantomData<(A, B, C, D, E)>);
pub struct Builder6<A, B, C, D, E, F>(PhantomData<(A, B, C, D, E, F)>);
pub struct Builder7<A, B, C, D, E, F, G>(PhantomData<(A, B, C, D, E, F, G)>);
pub struct Builder8<A, B, C, D, E, F, G, H>(PhantomData<(A, B, C, D, E, F, G, H)>);
pub struct Builder9<A, B, C, D, E, F, G, H, I>(PhantomData<(A, B, C, D, E, F, G, H, I)>);
pub struct Builder10<A, B, C, D, E, F, G, H, I, J>(PhantomData<(A, B, C, D, E, F, G, H, I, J)>);
pub struct Builder11<A, B, C, D, E, F, G, H, I, J, K>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K)>,
);
pub struct Builder12<A, B, C, D, E, F, G, H, I, J, K, L>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L)>,
);
pub struct Builder13<A, B, C, D, E, F, G, H, I, J, K, L, M>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L, M)>,
);
pub struct Builder14<A, B, C, D, E, F, G, H, I, J, K, L, M, N>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L, M, N)>,
);
pub struct Builder15<A, B, C, D, E, F, G, H, I, J, K, L, M, N, P>(
    PhantomData<(A, B, C, D, E, F, G, H, I, J, K, L, M, N, P)>,
);

impl Default for BuildStart {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildStart {
    pub fn new() -> Self {
        Self()
    }
    pub fn query(world: &World) -> QueryBorrow<'_, ()> {
        world.query()
    }
    pub fn query_mut(world: &mut World) -> QueryMut<'_, ()> {
        world.query_mut()
    }
    pub fn with<T>(self) -> Builder1<T> {
        Builder1(PhantomData)
    }
}

impl<T: Query> Builder1<T> {
    pub fn query<'a, 'world>(&'a self, world: &'world World) -> QueryBorrow<'world, T> {
        world.query()
    }
    pub fn query_mut<'a, 'world>(&'a self, world: &'world mut World) -> QueryMut<'world, T> {
        world.query_mut()
    }
    pub fn with<N>(self) -> Builder2<T, N> {
        Builder2(PhantomData)
    }
}

impl<A, B> Builder2<A, B>
where
    (A, B): Query,
{
    pub fn query<'a, 'world>(&'a self, world: &'world World) -> QueryBorrow<'world, (A, B)> {
        world.query()
    }
    pub fn query_mut<'a, 'world>(&'a self, world: &'world mut World) -> QueryMut<'world, (A, B)> {
        world.query_mut()
    }
    pub fn with<N>(self) -> Builder3<A, B, N> {
        Builder3(PhantomData)
    }
}

impl<A, B, C> Builder3<A, B, C>
where
    (A, B, C): Query,
{
    pub fn query(self, world: &World) -> QueryBorrow<'_, (A, B, C)> {
        world.query()
    }
    pub fn query_mut(self, world: &mut World) -> QueryMut<'_, (A, B, C)> {
        println!("QUERY!");
        world.query_mut()
    }
    //added for completions sake but.. not actually going to do much
    pub fn with<N>(self) -> Builder4<A, B, C, N>
    where
        (A, B, C, N): Query,
    {
        Builder4(PhantomData)
    }
}

use tealr::{
    mlu::{
        mlua::{Error, ToLua, UserData},
        TealData, TypedFunction,
    },
    new_type, TypeName,
};

#[derive(Clone, TypeName)]
pub struct LuaWorld(Arc<Mutex<World>>);

impl LuaWorld {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, hecs::World>, QueryError> {
        self.0.lock().map_err(QueryError::from)
    }
}

impl UserData for LuaWorld {
    fn add_methods<'lua, M: tealr::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
        <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
    }
}
impl TealData for LuaWorld {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_function(
            "start_query",
            |lua, function: tealr::mlu::mlua::Function| lua.scope(|v| Ok(())),
        )
    }
}

impl TypeName for BuildStart {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        new_type!(QueryBuilder)
    }
}
//for simplicity sake, lets get rid of the types for now.
impl<A> TypeName for Builder1<A> {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        new_type!(QueryBuilder1)
    }
}
//for simplicity sake, lets get rid of the types for now.
impl<A, B> TypeName for Builder2<A, B> {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        new_type!(QueryBuilder2)
    }
}

//for simplicity sake, lets get rid of the types for now.
impl<A, B, C> TypeName for Builder3<A, B, C> {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        new_type!(QueryBuilder3)
    }
}
impl UserData for BuildStart {
    fn add_methods<'lua, M: tealr::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
        <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
    }
}

#[derive(Debug)]
pub enum QueryError {
    PoisonedWorld,
}

impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The world mutex got poisoned. This shouldn't happen. PANIC!"
        )
    }
}
impl<T> From<PoisonError<T>> for QueryError {
    fn from(_: PoisonError<T>) -> Self {
        QueryError::PoisonedWorld
    }
}
impl From<QueryError> for Error {
    fn from(x: QueryError) -> Self {
        Error::external(x)
    }
}

#[derive(Clone, Copy, TypeName)]
pub struct LuaEntity(Entity);

impl TealData for LuaEntity {}
impl UserData for LuaEntity {
    fn add_methods<'lua, M: tealr::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
        <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
    }
}

impl std::error::Error for QueryError {}

impl TealData for BuildStart {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method(
            "query",
            |_,
             _,
             (luaworld, func): (
                LuaWorld,
                TypedFunction<LuaEntity, tealr::mlu::generics::X>,
            )| {
                let unlocked = luaworld.0.lock().map_err(QueryError::from)?;
                let mut x = BuildStart::query(&unlocked);
                let mut res = Vec::new();
                for (v, _) in &mut x {
                    res.push(func.call(LuaEntity(v))?)
                }
                //BuildStart::query()
                Ok(res)
            },
        );
        methods.add_method("with_integer", |_, this, ()| {
            Ok(this.to_owned().with::<&mut i64>())
        })
    }
}
impl<'a, T: 'a> UserData for Builder1<&'a mut T>
where
    &'a mut T: Query,
    T: ToOwned,
    <T as ToOwned>::Owned: ToLua<'static>,
{
    fn add_methods<'lua, M: tealr::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
        <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
    }
}
impl<'a, A: 'a> TealData for Builder1<&'a mut A>
where
    &'a mut A: Query,
    A: ToOwned,
    <A as ToOwned>::Owned: ToLua<'static>,
{
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("with_string", |_, this, ()| {
            let x = this.to_owned();
            let x = x.with::<&mut String>();
            Ok(x)
        })
    }
}

impl<'a, A: 'a, B: 'a> UserData for Builder2<&mut A, &mut B>
where
    (&'a mut A, &'a mut B): Query,
    A: ToOwned,
    B: ToOwned,
    <A as ToOwned>::Owned: ToLua<'static>,
    <B as ToOwned>::Owned: ToLua<'static>,
{
    fn add_methods<'lua, M: tealr::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
        <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
    }
}
impl<A, B> TealData for Builder2<A, B>
where
    (A, B): Query,
    A: ToOwned,
    B: ToOwned,
    <A as ToOwned>::Owned: ToLua<'static>,
    <B as ToOwned>::Owned: ToLua<'static>,
{
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method(
            "query",
            |lua,
             this,
             (world, func): (
                LuaWorld,
                tealr::mlu::TypedFunction<LuaEntity, tealr::mlu::generics::X>,
            )| {
                let z = world.lock()?;
                let mut x = this.query(&z);
                let mut res = Vec::new();
                for (v, x) in &mut x {
                    res.push(func.call(LuaEntity(v))?)
                }
                //BuildStart::query()
                Ok(res)
            },
        )
    }
}
