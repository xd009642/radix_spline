use radix_spline::RadixSpline;

#[repr(C)]
pub struct RadixSplineSearchBound {
    pub start: usize,
    pub stop: usize,
}

pub struct RadixSplineU64(RadixSpline<u64>);

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

pub struct RadixSplineU32(RadixSpline<u32>);

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
