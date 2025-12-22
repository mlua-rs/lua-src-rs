fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "lua51")]
    let version = lua_src::Lua51;
    #[cfg(feature = "lua52")]
    let version = lua_src::Lua52;
    #[cfg(feature = "lua53")]
    let version = lua_src::Lua53;
    #[cfg(feature = "lua54")]
    let version = lua_src::Lua54;
    #[cfg(feature = "lua55")]
    let version = lua_src::Lua55;

    let artifacts = lua_src::Build::new().build(version);
    artifacts.print_cargo_metadata();
}
