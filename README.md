# radix_spline

Implementation of RadixSpline ([paper](https://dl.acm.org/doi/10.1145/3401071.3401659)) 
in Rust. This is mainly a learning experiment for me but it is functional, as such
I believe it's fine for people to use if they so desire. The code is small though
so anyone depending on it should consider just vendoring the implementation.

A C API is provided via the radix_spline_c subproject.

## Performance

The synthetic 200M and Wikipedia 200M datasets on SOSD (Search On Sorted Data)
are used to benchmark. As the C API currently takes a contiguous array of keys
I modified the benchmark to do the same for the C++ version. I then disabled -ffast-math
and enabled -Ctarget-cpu=native on the Rust version so their two compiler flags
were equivalent. When ffast-math is enabled the C++ version maintains a firm lead.

Checking on the synthetic 200M entry dataset the results are:

![graph](https://raw.githubusercontent.com/xd009642/radix_spline/main/docs/uniform_dense_200M_uint64_rs_comparison.png)

The Rust implementation needs to go via a shared object and the C bindings have
to construct the object and box it added a layer of indirection. In comparison the
original implementation can benefit from inlining and lesser indirection from the
harness code. 

Contrastingly, I'm happy the index size is a consistent 8 bytes smaller (likely due to
padding).

Running on the 200M wikipedia dataset we can see the lookup latency isn't as clear:

![graph](https://raw.githubusercontent.com/xd009642/radix_spline/main/docs/wiki_ts_200M_uint64_rs_comparison.png)

As I continue working on this I'll be looking into more of what causes changes in
performance. I may need to create a comparative harness in Rust for running the Rust
benchmark or add some benchmark-useful C API methods.

## References

As well as the paper _RadixSpline: a single-pass learned index_, the
[reference C++ implementation](https://github.com/learnedsystems/RadixSpline)
was consulted. This project as a result is also licensed under the MIT license
although the code will likely diverge to be more Rusty.
