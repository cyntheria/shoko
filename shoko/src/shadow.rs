use std::cell::UnsafeCell;
use std::ptr::NonNull;

pub struct ShadowStack {
    storage: Vec<usize>,
    capacity: usize,
}

impl ShadowStack {
    pub fn new(capacity: usize) -> Self {
        Self {
            storage: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: usize) -> Result<(), &'static str> {
        if self.storage.len() >= self.capacity {
            return Err("Shadow stack overflow");
        }
        self.storage.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.storage.pop()
    }
}

pub struct SecureArena {
    buffer: UnsafeCell<Vec<u8>>,
    offset: UnsafeCell<usize>,
}

impl SecureArena {
    pub fn with_capacity(size: usize) -> Self {
        Self {
            buffer: UnsafeCell::new(vec![0; size]),
            offset: UnsafeCell::new(0),
        }
    }

    pub fn alloc(&self, size: usize) -> Option<NonNull<u8>> {
        unsafe {
            let buffer = &mut *self.buffer.get();
            let offset = &mut *self.offset.get();

            if *offset + size > buffer.len() {
                return None;
            }

            let ptr = buffer.as_mut_ptr().add(*offset);
            *offset += size;
            
            NonNull::new(ptr)
        }
    }

    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }
}
