use radix_spline::RadixSpline;

#[repr(C)]
pub struct RadixSplineSearchBound {
    pub start: usize,
    pub stop: usize,
}

pub struct RadixSplineU64(RadixSpline<u64>);

/// Finds the search bound for `key`.
///
/// # Safety
///
/// `spline` must be a live pointer returned by [`radix_spline_u64_build`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u64_find(
    spline: *const RadixSplineU64,
    key: u64,
) -> RadixSplineSearchBound {
    // SAFETY: The C API requires a live pointer returned by its build function.
    let spline = unsafe {
        spline
            .as_ref()
            .expect("radix spline pointer must not be null")
    };

    let range = spline.0.find(key);
    RadixSplineSearchBound {
        start: range.start,
        stop: range.end,
    }
}

/// Builds an index over a sorted array of keys.
///
/// # Safety
///
/// `keys` must point to `key_count` readable values for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u64_build(
    keys: *const u64,
    key_count: usize,
    radix_bits: u64,
    max_error: u64,
) -> *mut RadixSplineU64 {
    if keys.is_null() || key_count < 2 {
        return std::ptr::null_mut();
    }

    // SAFETY: The caller promises `keys` references `key_count` readable values.
    let keys = unsafe { std::slice::from_raw_parts(keys, key_count) };

    let mut builder = RadixSpline::builder(keys[0], keys[key_count - 1]);
    builder.radix_bits(radix_bits).max_error(max_error);
    builder.add_keys(keys.iter().copied());

    Box::into_raw(Box::new(RadixSplineU64(builder.build())))
}

/// Returns the allocated size of an index in bytes.
///
/// # Safety
///
/// `spline` must be a live pointer returned by [`radix_spline_u64_build`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u64_size(spline: *const RadixSplineU64) -> usize {
    // SAFETY: The C API requires a live pointer returned by its build function.
    unsafe {
        spline
            .as_ref()
            .expect("radix spline pointer must not be null")
    }
    .0
    .size_in_bytes()
}

/// Destroys an index, accepting null as a no-op.
///
/// # Safety
///
/// A non-null `spline` must have been returned by [`radix_spline_u64_build`] and
/// must not be used or destroyed again after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u64_destroy(spline: *mut RadixSplineU64) {
    if !spline.is_null() {
        // SAFETY: The pointer came from build and ownership is transferred once.
        drop(unsafe { Box::from_raw(spline) });
    }
}

pub struct RadixSplineU32(RadixSpline<u32>);

/// Finds the search bound for `key`.
///
/// # Safety
///
/// `spline` must be a live pointer returned by [`radix_spline_u32_build`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u32_find(
    spline: *const RadixSplineU32,
    key: u32,
) -> RadixSplineSearchBound {
    // SAFETY: The C API requires a live pointer returned by its build function.
    let spline = unsafe {
        spline
            .as_ref()
            .expect("radix spline pointer must not be null")
    };

    let range = spline.0.find(key);
    RadixSplineSearchBound {
        start: range.start,
        stop: range.end,
    }
}

/// Builds an index over a sorted array of keys.
///
/// # Safety
///
/// `keys` must point to `key_count` readable values for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u32_build(
    keys: *const u32,
    key_count: usize,
    radix_bits: u64,
    max_error: u32,
) -> *mut RadixSplineU32 {
    if keys.is_null() || key_count < 2 {
        return std::ptr::null_mut();
    }

    // SAFETY: The caller promises `keys` references `key_count` readable values.
    let keys = unsafe { std::slice::from_raw_parts(keys, key_count) };

    let mut builder = RadixSpline::builder(keys[0], keys[key_count - 1]);
    builder.radix_bits(radix_bits).max_error(max_error);
    builder.add_keys(keys.iter().copied());

    Box::into_raw(Box::new(RadixSplineU32(builder.build())))
}

/// Returns the allocated size of an index in bytes.
///
/// # Safety
///
/// `spline` must be a live pointer returned by [`radix_spline_u32_build`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u32_size(spline: *const RadixSplineU32) -> usize {
    // SAFETY: The C API requires a live pointer returned by its build function.
    unsafe {
        spline
            .as_ref()
            .expect("radix spline pointer must not be null")
    }
    .0
    .size_in_bytes()
}

/// Destroys an index, accepting null as a no-op.
///
/// # Safety
///
/// A non-null `spline` must have been returned by [`radix_spline_u32_build`] and
/// must not be used or destroyed again after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn radix_spline_u32_destroy(spline: *mut RadixSplineU32) {
    if !spline.is_null() {
        // SAFETY: The pointer came from build and ownership is transferred once.
        drop(unsafe { Box::from_raw(spline) });
    }
}
