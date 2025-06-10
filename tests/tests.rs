//! This is standalone tests that executed at runtime (not during build)

#[test]
fn test_build() {
    let host = target_lexicon::Triple::host().to_string();
    let outdir = tempfile::tempdir().expect("Failed to create temp dir");
    let mut build = lua_src::Build::new();
    build.target(&host).out_dir(&outdir);

    for version in [
        lua_src::Lua51,
        lua_src::Lua52,
        lua_src::Lua53,
        lua_src::Lua54,
    ] {
        let _artifacts = build.build(version);
    }
}
