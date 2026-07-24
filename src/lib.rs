//! stages:
//!
//! 1. Build spline (params: max_error)
//! 2. Build radix table (params: radix_bits)
//! 3. ???

pub struct RadixSpline {
    min_key: u64,
    max_key: u64,
    current_key_count: usize,
    radix_bits: u64,
    shift_bits: u64,
    max_error: u64,
    radix_table: Vec<u32>,
    spline_points: Vec<u64>,
}

pub struct RadixSplineBuilder {
    min_key: u64,
    max_key: u64,
    radix_bits: u64,
    shift_bits: u64,
    max_error: u64,

    previous_key: u64,

    radix_table: Vec<u32>,
    // TODO generic coord type
    spline_points: Vec<u64>,
    current_key_count: usize,
}

impl RadixSplineBuilder {
    pub fn new(min_key: u64, max_key: u64) -> Self {
        let radix_bits = 18;
        let shift_bits = num_shift_bits(max_key - min_key, radix_bits);
        let radix_table_capacity = ((max_key - min_key) >> shift_bits) as usize;
        Self {
            min_key,
            max_key,
            previous_key: min_key,
            radix_bits,
            shift_bits,
            max_error: 32,
            radix_table: Vec::with_capacity(radix_table_capacity),
            spline_points: vec![],
            current_key_count: 0,
        }
    }

    pub fn max_error(&mut self, max_error: u64) -> &mut Self {
        if self.current_key_count > 0 {
            panic!("Cannot change radix key after construction has started");
        }
        self.max_error = max_error;
        self
    }

    pub fn radix_bits(&mut self, radix_bits: u64) -> &mut Self {
        if self.current_key_count > 0 {
            panic!("Cannot change radix key after construction has started");
        }
        self.radix_bits = radix_bits;
        self.shift_bits = num_shift_bits(self.max_key - self.min_key, radix_bits);
        let radix_table_capacity = ((self.max_key - self.min_key) >> self.shift_bits) as usize;
        self.radix_table.reserve(radix_table_capacity);
        self
    }

    pub fn add_keys(&mut self, mut it: impl Iterator<Item=u64>) -> &mut Self {
        for key in it {
            self.add_key(key);
        }
        self
    }

    pub fn add_key(&mut self, key: u64) -> &mut Self {
        assert!(key >= self.min_key);
        assert!(key <= self.max_key);
        assert!(key >= self.previous_key);
        self.add_key_to_spline(key);
        
        self.previous_key = key;
        self.current_key_count += 1;

        self
    }

    // GreedySplineCorridor implementation
    fn add_key_to_spline(&mut self, key: u64) {
        if key == self.previous_key {
            // Nothing to do, the key is the same on the curve
            return;
        }
        todo!()
    }

    pub fn build(self) -> RadixSpline {
        RadixSpline {
            min_key: self.min_key,
            max_key: self.max_key,
            current_key_count: self.current_key_count,
            radix_bits: self.radix_bits,
            shift_bits: self.shift_bits,
            max_error: self.max_error,
            radix_table: self.radix_table,
            spline_points: self.spline_points,
        }
    }
}

fn num_shift_bits(diff: u64, radix_bits: u64) -> u64 {
    let leading_zeros = diff.leading_zeros() as u64;
    if 64 - leading_zeros < radix_bits {
        0
    } else {
        64 - radix_bits - leading_zeros
    }
}
