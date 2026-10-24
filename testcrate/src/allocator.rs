use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;

// Rust and WASI libc must share an allocator
struct WasiAllocator;

unsafe extern "C" {
    fn posix_memalign(ptr: *mut *mut c_void, align: usize, size: usize) -> i32;
    fn free(ptr: *mut c_void);
}

unsafe impl GlobalAlloc for WasiAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut ptr = std::ptr::null_mut();
        if unsafe {
            posix_memalign(
                &mut ptr,
                layout.align().max(size_of::<usize>()),
                layout.size(),
            )
        } != 0
        {
            return std::ptr::null_mut();
        }
        ptr.cast()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        unsafe { free(ptr.cast()) };
    }
}

#[global_allocator]
static ALLOCATOR: WasiAllocator = WasiAllocator;
