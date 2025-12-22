#![allow(unsafe_op_in_unsafe_fn, clippy::missing_safety_doc)]

use std::os::raw::{c_char, c_int, c_long, c_void};

unsafe extern "C" {
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
    #[cfg(any(feature = "lua53", feature = "lua54", feature = "lua55"))]
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
    #[cfg(any(feature = "lua53", feature = "lua54", feature = "lua55"))]
    pub fn lua_getglobal(state: *mut c_void, k: *const c_char) -> c_int;
}

#[cfg(feature = "lua51")]
pub unsafe fn lua_getglobal(state: *mut c_void, k: *const c_char) {
    lua_getfield(state, -10002 /* LUA_GLOBALSINDEX */, k);
}

#[cfg(not(feature = "lua51"))]
pub unsafe fn lua_pcall(
    state: *mut c_void,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    lua_pcallk(state, nargs, nresults, errfunc, 0, std::ptr::null())
}

pub unsafe fn to_string<'a>(state: *mut c_void, index: c_int) -> &'a str {
    let mut len: c_long = 0;
    let ptr = lua_tolstring(state, index, &mut len);
    let bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    std::str::from_utf8(bytes).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_version() {
        unsafe {
            let state = luaL_newstate();
            assert!(!state.is_null());

            luaL_openlibs(state);

            lua_getglobal(state, c"_VERSION".as_ptr());
            let version = to_string(state, -1);

            #[cfg(feature = "lua51")]
            assert_eq!(version, "Lua 5.1");
            #[cfg(feature = "lua52")]
            assert_eq!(version, "Lua 5.2");
            #[cfg(feature = "lua53")]
            assert_eq!(version, "Lua 5.3");
            #[cfg(feature = "lua54")]
            assert_eq!(version, "Lua 5.4");
            #[cfg(feature = "lua55")]
            assert_eq!(version, "Lua 5.5");
        }
    }

    #[test]
    fn test_unicode_identifiers() {
        unsafe {
            let state = luaL_newstate();
            let ret = luaL_loadstring(state, c"😀 = '🌚︎'".as_ptr());

            #[cfg(any(feature = "lua54", feature = "lua55"))]
            {
                assert_eq!(ret, 0);
                assert_eq!(lua_pcall(state, 0, 0, 0), 0);
                lua_getglobal(state, c"😀".as_ptr());
                assert_eq!(to_string(state, -1), "🌚︎");
            }

            #[cfg(not(any(feature = "lua54", feature = "lua55")))]
            assert_ne!(ret, 0);
        }
    }

    #[test]
    fn test_exceptions() {
        unsafe {
            let state = luaL_newstate();
            assert!(!state.is_null());

            unsafe extern "C-unwind" fn it_panics(state: *mut c_void) -> c_int {
                luaL_error(state, c"exception!".as_ptr())
            }

            lua_pushcclosure(state, it_panics, 0);
            let result = lua_pcall(state, 0, 0, 0);
            assert_eq!(result, 2); // LUA_ERRRUN
            assert_eq!(to_string(state, -1), "exception!");
        }
    }
}
