use nix::sys::mman::{mmap_anonymous, munmap, MapFlags, ProtFlags};
use std::num::NonZeroUsize;
use std::ffi::c_void;
use std::ptr::NonNull;

pub fn create_executable_buffer(size: usize) -> Result<*mut c_void, Box<dyn std::error::Error>> {
    let len = NonZeroUsize::new(size)
        .ok_or("Memory size must be greater than zero")?;

    unsafe {
        let prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC;
        let flags = MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS;

        let ptr_non_null = mmap_anonymous(
            None,
            len,
            prot,
            flags,
        )?;

        Ok(ptr_non_null.as_ptr())
    }
}

pub fn free_executable_buffer(ptr: *mut c_void, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let ptr_non_null = NonNull::new(ptr)
        .ok_or("Cannot free a null pointer")?;

    unsafe {
        munmap(ptr_non_null, size)?;
    }
    Ok(())
}
