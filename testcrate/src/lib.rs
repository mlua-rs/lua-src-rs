use std::os::raw::{c_char, c_int, c_long, c_void};

extern "C" {
    pub fn luaL_newstate() -> *mut c_void;
    pub fn luaL_openlibs(state: *mut c_void);
    pub fn lua_getfield(state: *mut c_void, index: c_int, k: *const c_char);
    pub fn lua_tolstring(state: *mut c_void, index: c_int, len: *mut c_long) -> *const c_char;
    pub fn luaL_loadstring(state: *mut c_void, s: *const c_char) -> c_int;
    pub fn luaL_error(state: *mut c_void, fmt: *const c_char, ...) -> c_int;

    pub fn lua_pushcclosure(
        state: *mut c_void,
        f: unsafe extern "C-unwind" fn(state: *mut c_void) -> c_int,
        n: c_int,
    );

    #[cfg(feature = "lua51")]
    pub fn lua_pcall(state: *mut c_void, nargs: c_int, nresults: c_int, errfunc: c_int) -> c_int;
    #[cfg(feature = "lua52")]
    pub fn lua_pcallk(
        state: *mut c_void,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: c_int,
        k: *const c_void,
    ) -> c_int;
    #[cfg(any(feature = "lua53", feature = "lua54"))]
    pub fn lua_pcallk(
        state: *mut c_void,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: isize,
        k: *const c_void,
    ) -> c_int;

    #[cfg(feature = "lua52")]
    pub fn lua_getglobal(state: *mut c_void, k: *const c_char);
    #[cfg(any(feature = "lua53", feature = "lua54"))]
    pub fn lua_getglobal(state: *mut c_void, k: *const c_char) -> c_int;
}

#[cfg(feature = "lua51")]
pub unsafe fn lua_getglobal(state: *mut c_void, k: *const c_char) {
    lua_getfield(state, -10002 /* LUA_GLOBALSINDEX */, k);
}

#[cfg(any(feature = "lua52", feature = "lua53", feature = "lua54"))]
pub unsafe fn lua_pcall(
    state: *mut c_void,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    lua_pcallk(state, nargs, nresults, errfunc, 0, std::ptr::null())
}

#[test]
fn test_lua() {
    use std::{ptr, slice};
    unsafe {
        let state = luaL_newstate();
        assert!(state != ptr::null_mut());

        luaL_openlibs(state);

        let version = {
            lua_getglobal(state, "_VERSION\0".as_ptr().cast());
            let mut len: c_long = 0;
            let version_ptr = lua_tolstring(state, -1, &mut len);
            slice::from_raw_parts(version_ptr as *const u8, len as usize)
        };

        #[cfg(feature = "lua51")]
        assert_eq!(version, "Lua 5.1".as_bytes());
        #[cfg(feature = "lua52")]
        assert_eq!(version, "Lua 5.2".as_bytes());
        #[cfg(feature = "lua53")]
        assert_eq!(version, "Lua 5.3".as_bytes());
        #[cfg(feature = "lua54")]
        assert_eq!(version, "Lua 5.4".as_bytes());
    }
}

#[test]
fn test_unicode_identifiers() {
    unsafe {
        let state = luaL_newstate();
        let code = "local 😀 = 0\0";
        let ret = luaL_loadstring(state, code.as_ptr().cast());
        #[cfg(feature = "lua54")]
        assert_eq!(0, ret);
        #[cfg(not(feature = "lua54"))]
        assert_ne!(0, ret);
    }
}

#[test]
fn test_exceptions() {
    use std::{ptr, slice, str};
    unsafe {
        let state = luaL_newstate();
        assert!(state != ptr::null_mut());

        unsafe extern "C-unwind" fn it_panics(state: *mut c_void) -> c_int {
            luaL_error(state, "exception!\0".as_ptr().cast())
        }

        lua_pushcclosure(state, it_panics, 0);
        let result = lua_pcall(state, 0, 0, 0);
        assert_eq!(result, 2); // LUA_ERRRUN
        let s = {
            let mut len: c_long = 0;
            let version_ptr = lua_tolstring(state, -1, &mut len);
            let s = slice::from_raw_parts(version_ptr as *const u8, len as usize);
            str::from_utf8(s).unwrap()
        };
        assert_eq!(s, "exception!");
    }
}
