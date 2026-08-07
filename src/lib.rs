//! stages:
//!
//! 1. Build spline (params: max_error)
//! 2. Build radix table (params: radix_bits)
//! 3. ???

use std::ops::Range;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Orientation {
    Clockwise,
    AntiClockwise,
    Colinear
}

pub struct RadixSpline {
    min_key: u64,
    max_key: u64,
    current_key_count: usize,
    radix_bits: u64,
    shift_bits: u64,
    max_error: f32, // Was f64 in original impl
    radix_table: Vec<u32>,
    spline_points: Vec<(u64, f32)>,
}

impl RadixSpline {
    pub fn builder(min_key: u64, max_key: u64) -> RadixSplineBuilder {
        RadixSplineBuilder::new(min_key, max_key)
    }

    pub fn find(&self, key: u64) -> Range<usize> {
        todo!()
    }
}

pub struct RadixSplineBuilder {
    min_key: u64,
    max_key: u64,
    radix_bits: u64,
    shift_bits: u64,
    max_error: f32,

    previous_key: u64,
    previous_position: u64,
    previous_point: (u64, f32),
    previous_prefix: u64,

    radix_table: Vec<u32>,
    // TODO generic coord type
    spline_points: Vec<(u64, f32)>,
    current_key_count: usize,
    distinct_key_count: usize,
    upper_limit: (u64, f32),
    lower_limit: (u64, f32),
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
            previous_prefix: 0,
            radix_bits,
            shift_bits,
            max_error: 32.0,
            radix_table: vec![0; radix_table_capacity],
            spline_points: vec![],
            current_key_count: 0,
            distinct_key_count: 0,
            upper_limit: (0, 0.0),
            lower_limit: (0, 0.0),
            previous_point: (0, 0.0),
            previous_position: 0,
        }
    }

    pub fn max_error(&mut self, max_error: u64) -> &mut Self {
        if self.current_key_count > 0 {
            panic!("Cannot change radix key after construction has started");
        }
        self.max_error = max_error as f32;
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

    pub fn add_keys(&mut self, it: impl Iterator<Item=u64>) -> &mut Self {
        for (pos, key) in it.enumerate() {
            self.add_key(key, pos);
        }
        self
    }

    pub fn add_key(&mut self, key: u64, position: usize) -> &mut Self {
        assert!(key >= self.min_key);
        assert!(key <= self.max_key);
        assert!(key >= self.previous_key);
        self.maybe_add_key_to_spline(key, position as f32);
        
        self.previous_key = key;
        self.current_key_count += 1;
        self.previous_position = position as u64;

        self
    }

    // GreedySplineCorridor implementation
    fn maybe_add_key_to_spline(&mut self, key: u64, position: f32) {
        if self.current_key_count == 0 {
            
            self.distinct_key_count += 1;
            self.add_key_to_spline(key, position);
            self.previous_point = (key, position);
            return;
        }
        if key == self.previous_key {
            // Nothing to do, the key is the same on the curve
            return;
        }
        self.distinct_key_count += 1;

        if self.distinct_key_count == 2 {
            self.upper_limit = (key, position + self.max_error);
            let lower = (position - self.max_error).max(0.0);
            self.lower_limit = (key, lower);
            self.previous_point = (key, position);
            return;
        }

        let (last_point, last_distance) = self.spline_points.last().copied().unwrap();

        let upper_y = position + self.max_error;
        let lower_y = (position - self.max_error).max(0.0);

        assert!(self.upper_limit.0 >= last_point);
        assert!(self.lower_limit.0 >= last_point);
        assert!(key >= last_point);

        let upper_limit_dx = (self.upper_limit.0 - last_point) as f32;
        let lower_limit_dx = (self.lower_limit.0 - last_point) as f32;
        let dx = key - last_point;

        assert!(self.upper_limit.1 >= last_distance);
        assert!(position >= last_distance);

        let upper_limit_dy = (self.upper_limit.1 - last_distance) as f32;
        let lower_limit_dy = (self.lower_limit.1 - last_distance) as f32;
        let dy = position - last_distance;

        let upper_limit = (upper_limit_dx, upper_limit_dy);
        let lower_limit = (lower_limit_dx, lower_limit_dy);
        let delta = (dx as f32, dy);

        assert_ne!(self.previous_point.0, last_point);

        if compute_orientation(upper_limit, delta) != Orientation::Clockwise || compute_orientation(lower_limit, delta) != Orientation::AntiClockwise {
            let (key, dist) = self.previous_point;
            self.add_key_to_spline(key, dist);
            self.upper_limit = (key, upper_y);
            self.lower_limit = (key, lower_y);
        } else {
            assert!(upper_y >= last_distance);
            let upper_dy = upper_y - last_distance;
            if compute_orientation(upper_limit, (dx as f32, upper_dy)) == Orientation::Clockwise {
                self.upper_limit = (key, upper_dy);
            }

            let lower_dy = lower_y - last_distance;
            if compute_orientation(lower_limit, (dx as f32, lower_dy)) == Orientation::AntiClockwise {
                self.lower_limit = (key, lower_dy);
            }
        }
        self.previous_point = (key, position);
    }

    fn add_key_to_spline(&mut self, key: u64, position: f32) {
        self.spline_points.push((key, position));
        self.maybe_add_key_to_radix_table(key);
    }

    fn maybe_add_key_to_radix_table(&mut self, key: u64) {
        let curr_prefix = (key - self.min_key) >> self.shift_bits;
        assert!(curr_prefix < self.radix_table.len() as u64);
        if curr_prefix != self.previous_prefix {
            let index = self.spline_points.len() - 1;
            for i in (self.previous_prefix + 1)..curr_prefix {
                self.radix_table[i as usize] = index as u32;
            }
        }
    }

    fn finalize(&mut self) {
        assert!(self.current_key_count == 0 || self.previous_key == self.max_key);
        if self.current_key_count > 0 && self.spline_points.last().copied().unwrap().0 != self.previous_key {
            self.add_key_to_spline(self.previous_key, self.previous_position as f32);
        }
    
        let num_spline_points = self.spline_points.len() as u32;
        for i in (self.previous_prefix as usize)..self.radix_table.len() {
            self.radix_table[i] = num_spline_points;
        }
    }

    pub fn build(mut self) -> RadixSpline {
    
        self.finalize();

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

fn compute_orientation(p1: (f32, f32), p2: (f32, f32)) -> Orientation {
    let expr = p1.1.mul_add(p2.0, - (p2.1 * p1.0));
    if expr > f32::EPSILON {
        Orientation::Clockwise
    } else if expr < -f32::EPSILON {
        Orientation::AntiClockwise
    } else {
        Orientation::Colinear
    }
}
