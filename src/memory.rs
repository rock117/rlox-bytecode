use libc;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
pub fn GROW_CAPACITY(old_capacity: usize) -> usize {
    if old_capacity < 8 {
        8
    } else {
        8 * 2
    }
}

pub fn GROW_ARRAY<T>(pointer: *mut c_void, old_count: usize, new_count: usize) -> *mut c_void {
    unsafe {
        reallocate(
            pointer,
            size_of::<T>() * old_count,
            size_of::<T>() * new_count,
        )
    }
}

// #define FREE_ARRAY(type, pointer, oldCount) reallocate(pointer, sizeof(type) * (oldCount), 0)
pub fn FREE_ARRAY<T>(pointer: *mut c_void, old_count: usize) -> *mut c_void {
    unsafe { reallocate(pointer, size_of::<T>() * old_count, 0) }
}

unsafe fn reallocate(pointer: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void {
    if new_size == 0 {
        libc::free(pointer);
        return null_mut();
    }
    let result = libc::realloc(pointer, new_size);
    if result.is_null() {
        std::process::exit(1);
    }
    return result;
}
