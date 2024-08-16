//! Bindings to the `native_vsync` library on OpenHarmony
//!
//! This library can be used to receive callbacks on vsync signals

use core::ffi::{c_int, c_void};

use log::debug;
use ohos_sys::vsync::{
    OH_NativeVSync, OH_NativeVSync_Create, OH_NativeVSync_Destroy, OH_NativeVSync_FrameCallback,
    OH_NativeVSync_GetPeriod, OH_NativeVSync_RequestFrame,
};

mod log;

pub struct NativeVsync {
    raw: *mut OH_NativeVSync,
}

#[derive(Debug)]
pub enum NativeVsyncError {
    InvalidArgs,
    CreateFailed,
    RawErr(c_int),
}

impl NativeVsync {
    pub fn new(name: &str) -> Result<Self, NativeVsyncError> {
        let name_len: u32 = name
            .len()
            .try_into()
            .map_err(|_| NativeVsyncError::InvalidArgs)?;
        // SAFETY: OH_NativeVSync_Create will not attempt to modify the string.
        // The official example also shows usage of a name without a trailing `\0`
        let raw = unsafe { OH_NativeVSync_Create(name.as_ptr().cast(), name_len) };
        Ok(NativeVsync { raw })
    }

    /// Create from a raw pointer to a valid `OH_NativeVSync` instance.
    ///
    /// # Safety
    ///
    /// `native_vsync` must be a valid, live OH_NativeVSync instance.
    /// The ownership of `native_vsync` must be exclusive and is transferred to the new object.
    pub unsafe fn from_raw(native_vsync: *mut OH_NativeVSync) -> Self {
        debug_assert!(!native_vsync.is_null());
        debug_assert!(native_vsync.is_aligned());
        Self { raw: native_vsync }
    }

    /// Returns the refernece to the raw OH_NativeVSync and consumes self
    ///
    /// `NativeVsync::from_raw` can be used to reconstruct Self later.
    /// This can be used to pass the owned NativeVsync object to the callback function.
    pub fn into_raw(self) -> *mut OH_NativeVSync {
        let raw = self.raw;
        core::mem::forget(self);
        raw
    }

    /// Request a Callback to `callback` on the next Vsync frame
    ///
    /// `data` will be passed to the callback.
    ///
    /// # Safety
    ///
    /// If data is used in the callback then data must live long enough and be ThreadSafe to use.
    /// Todo: Define the requirements more clearly.
    pub unsafe fn request_raw_callback(
        &self,
        callback: OH_NativeVSync_FrameCallback,
        data: *mut c_void,
    ) -> Result<(), NativeVsyncError> {
        let res = unsafe { OH_NativeVSync_RequestFrame(self.raw, callback, data) };
        if res == 0 {
            Ok(())
        } else {
            Err(NativeVsyncError::RawErr(res))
        }
    }

    pub unsafe fn request_raw_callback_with_self(
        self,
        callback: OH_NativeVSync_FrameCallback,
    ) -> Result<(), NativeVsyncError> {
        let res =
            unsafe { OH_NativeVSync_RequestFrame(self.raw, callback, self.raw as *mut c_void) };
        if res == 0 {
            core::mem::forget(self);
            Ok(())
        } else {
            // implicit drop / destroy
            Err(NativeVsyncError::RawErr(res))
        }
    }

    /// Returns the vsync period in nanoseconds.
    pub fn get_period(&self) -> Result<u64, NativeVsyncError> {
        let period = unsafe {
            let mut period: i64 = -1;
            let res = OH_NativeVSync_GetPeriod(self.raw, (&mut period) as *mut i64);
            if res == 0 {
                debug_assert!(period > 0, "Period must be a positive non-zero integer");
                period as u64
            } else {
                debug!("OH_NativeVSync_GetPeriod failed with {res}");
                return Err(NativeVsyncError::RawErr(res));
            }
        };
        Ok(period)
    }
}

impl Drop for NativeVsync {
    fn drop(&mut self) {
        // SAFETY: We never leaked the pointer, so we are sure we still own the
        // nativeVsync object and can destroy it.
        unsafe { OH_NativeVSync_Destroy(self.raw) };
    }
}
