// SPDX-License-Identifier: GPL-2.0

//! Synchronization primitives

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Spinlock implementation
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Spinlock<T> {}
unsafe impl<T: Send> Send for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    
    pub fn lock(&self) -> SpinlockGuard<T> {
        while self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            // Busy wait
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        
        SpinlockGuard { lock: self }
    }
    
    pub fn try_lock(&self) -> Option<SpinlockGuard<T>> {
        if self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            Some(SpinlockGuard { lock: self })
        } else {
            None
        }
    }
}

pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

/// Mutex implementation (placeholder - would need proper scheduler integration)
pub struct Mutex<T> {
    inner: Spinlock<T>,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: Spinlock::new(data),
        }
    }
    
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            guard: self.inner.lock(),
        }
    }
}

pub struct MutexGuard<'a, T> {
    guard: SpinlockGuard<'a, T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

/// RwLock implementation (placeholder)
pub struct RwLock<T> {
    inner: Spinlock<T>,
}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: Spinlock::new(data),
        }
    }
    
    pub fn read(&self) -> RwLockReadGuard<T> {
        RwLockReadGuard {
            guard: self.inner.lock(),
        }
    }
    
    pub fn write(&self) -> RwLockWriteGuard<T> {
        RwLockWriteGuard {
            guard: self.inner.lock(),
        }
    }
}

pub struct RwLockReadGuard<'a, T> {
    guard: SpinlockGuard<'a, T>,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

pub struct RwLockWriteGuard<'a, T> {
    guard: SpinlockGuard<'a, T>,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}
