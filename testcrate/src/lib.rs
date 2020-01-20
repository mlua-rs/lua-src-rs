use std::ffi::c_void;
use std::ptr;

extern "C" {
    pub fn luaL_newstate() -> *mut c_void;
}

#[test]
fn newstate_works() {
    unsafe {
        assert!(luaL_newstate() != ptr::null_mut());
    }
}
