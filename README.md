# radix_spline

Implementation of RadixSpline ([paper](https://dl.acm.org/doi/10.1145/3401071.3401659)) 
in Rust. 

This is currently, a work-in-progress for my own fun/learnings.

A C API is provided via the radix_spline_c subproject.

## Performance

Initial checks on performance show it as as similar to the C++ implementation 
on the _Search On Sorted Data_ (SOSD) benchmark. Checking on the synthetic
200M entry dataset the results are:

![graph](./docs/uniform_dense_200M_uint64_rs_comparison.png)

The Rust implementation needs to go via a shared object and the C bindings have
to construct the object and box it added a layer of indirection. In comparison the
original implementation can benefit from inlining and lesser indirection from the
harness code. Because of this I'm not overly concerned with the gap in lookup latency.

Contrastingly, I'm happy the index size is a consistent 8 bytes smaller (likely due to
padding), and the index construction speed measures as significantly faster!

Running on the 200M wikipedia dataset we can see the lookup latency isn't as clear:

![graph](./docs/wiki_ts_200M_uint64_rs_comparison.png)

As I continue working on this I'll be looking into more of what causes changes in
performance.

## References

As well as the paper _RadixSpline: a single-pass learned index_, the
[reference C++ implementation](https://github.com/learnedsystems/RadixSpline)
was consulted. This project as a result is also licensed under the MIT license
although the code will likely diverge to be more Rusty.
