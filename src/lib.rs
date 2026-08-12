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
    Colinear,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RadixSpline<K> {
    min_key: K,
    max_key: K,
    current_key_count: usize,
    shift_bits: usize,
    max_error: f64,
    radix_table: Vec<u32>,
    spline_points: Vec<(K, f64)>,
}

impl<K> RadixSpline<K>
where
    K: Key,
{
    pub fn builder(min_key: K, max_key: K) -> RadixSplineBuilder<K> {
        RadixSplineBuilder::new(min_key, max_key)
    }

    pub fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.radix_table.capacity() * std::mem::size_of::<u32>()
            + self.spline_points.capacity() * std::mem::size_of::<(K, f64)>()
    }

    fn estimated_position(&self, key: K) -> f64 {
        if key <= self.min_key {
            0.0
        } else if key >= self.max_key {
            (self.current_key_count - 1) as f64
        } else {
            let index = self.get_spline_segment(key);
            let down = self.spline_points[index - 1];
            let up = self.spline_points[index];

            let dx = (up.0 - down.0).to_f64();
            let dy = up.1 - down.1;
            let slope = dy / dx;

            let dk = (key - down.0).to_f64();
            dk.mul_add(slope, down.1)
        }
    }

    fn get_spline_segment(&self, key: K) -> usize {
        let prefix = ((key - self.min_key) >> self.shift_bits)
            .to_usize()
            .expect("radix prefix does not fit in usize");
        assert!(prefix + 1 < self.radix_table.len());

        let begin = self.radix_table[prefix] as usize;
        let end = self.radix_table[prefix + 1] as usize;

        if end - begin < 32 {
            self.spline_points[begin..end]
                .iter()
                .position(|x| x.0 >= key)
                .map(|x| begin + x)
                .unwrap_or(end)
        } else {
            begin + self.spline_points[begin..end].partition_point(|x| x.0 < key)
        }
    }

    pub fn find(&self, key: K) -> Range<usize> {
        let est_pos = self.estimated_position(key);
        let begin = (est_pos - self.max_error).max(0.0) as usize;
        let end = ((est_pos + self.max_error) as usize + 2).min(self.current_key_count);
        begin..end
    }
}

pub struct RadixSplineBuilder<Key> {
    min_key: Key,
    max_key: Key,
    radix_bits: u64,
    shift_bits: usize,
    max_error: f64,

    previous_key: Key,
    previous_position: u64,
    previous_point: (Key, f64),
    previous_prefix: usize,

    radix_table: Vec<u32>,
    spline_points: Vec<(Key, f64)>,
    current_key_count: usize,
    distinct_key_count: usize,
    upper_limit: (Key, f64),
    lower_limit: (Key, f64),
}

impl<K> RadixSplineBuilder<K>
where
    K: Key,
{
    pub fn new(min_key: K, max_key: K) -> Self {
        assert!(min_key < max_key);
        let radix_bits = 18;
        let shift_bits = num_shift_bits(max_key - min_key, radix_bits);
        let radix_table_capacity = ((max_key - min_key) >> shift_bits)
            .to_usize()
            .expect("radix table size does not fit in usize")
            + 2;
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
            upper_limit: (K::zero(), 0.0),
            lower_limit: (K::zero(), 0.0),
            previous_point: (K::zero(), 0.0),
            previous_position: 0,
        }
    }

    pub fn max_error(&mut self, max_error: K) -> &mut Self {
        if self.current_key_count > 0 {
            panic!("Cannot change radix key after construction has started");
        }
        self.max_error = max_error.to_f64();
        self
    }

    pub fn radix_bits(&mut self, radix_bits: u64) -> &mut Self {
        if self.current_key_count > 0 {
            panic!("Cannot change radix key after construction has started");
        }
        self.radix_bits = radix_bits;
        self.shift_bits = num_shift_bits(self.max_key - self.min_key, radix_bits);
        let radix_table_capacity = ((self.max_key - self.min_key) >> self.shift_bits)
            .to_usize()
            .expect("radix table size does not fit in usize");
        self.radix_table.resize(radix_table_capacity + 2, 0);
        self
    }

    pub fn add_keys(&mut self, it: impl Iterator<Item = K>) -> &mut Self {
        for key in it {
            self.add_key(key);
        }
        self
    }

    pub fn add_key(&mut self, key: K) -> &mut Self {
        assert!(key >= self.min_key);
        assert!(key <= self.max_key);
        assert!(key >= self.previous_key);

        let position = self.current_key_count;

        self.maybe_add_key_to_spline(key, position as f64);

        self.previous_key = key;
        self.current_key_count += 1;
        self.previous_position = position as u64;

        self
    }

    // GreedySplineCorridor implementation
    fn maybe_add_key_to_spline(&mut self, key: K, position: f64) {
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

        let upper_limit_dx = (self.upper_limit.0 - last_point).to_f64();
        let lower_limit_dx = (self.lower_limit.0 - last_point).to_f64();
        let dx = key - last_point;

        assert!(self.upper_limit.1 >= last_distance);
        assert!(position >= last_distance);

        let upper_limit_dy = self.upper_limit.1 - last_distance;
        let lower_limit_dy = self.lower_limit.1 - last_distance;
        let dy = position - last_distance;

        let upper_limit = (upper_limit_dx, upper_limit_dy);
        let lower_limit = (lower_limit_dx, lower_limit_dy);
        let delta = (dx.to_f64(), dy);

        assert_ne!(self.previous_point.0, last_point);

        if compute_orientation(upper_limit, delta) != Orientation::Clockwise
            || compute_orientation(lower_limit, delta) != Orientation::AntiClockwise
        {
            let (prev_key, dist) = self.previous_point;
            self.add_key_to_spline(prev_key, dist);
            self.upper_limit = (key, upper_y);
            self.lower_limit = (key, lower_y);
        } else {
            assert!(upper_y >= last_distance);
            let upper_dy = upper_y - last_distance;
            if compute_orientation(upper_limit, (dx.to_f64(), upper_dy)) == Orientation::Clockwise {
                self.upper_limit = (key, upper_y);
            }

            let lower_dy = lower_y - last_distance;
            if compute_orientation(lower_limit, (dx.to_f64(), lower_dy))
                == Orientation::AntiClockwise
            {
                self.lower_limit = (key, lower_y);
            }
        }
        self.previous_point = (key, position);
    }

    fn add_key_to_spline(&mut self, key: K, position: f64) {
        self.spline_points.push((key, position));
        self.maybe_add_key_to_radix_table(key);
    }

    fn maybe_add_key_to_radix_table(&mut self, key: K) {
        let curr_prefix = ((key - self.min_key) >> self.shift_bits)
            .to_usize()
            .expect("radix prefix does not fit in usize");
        assert!(
            curr_prefix < self.radix_table.len(),
            "radix table contains {} elements but prefix is {}",
            self.radix_table.len(),
            curr_prefix
        );
        if curr_prefix != self.previous_prefix {
            let index = u32::try_from(self.spline_points.len() - 1)
                .expect("radix spline contains more spline points than u32 can index");
            for i in (self.previous_prefix + 1)..=curr_prefix {
                self.radix_table[i] = index;
            }
            self.previous_prefix = curr_prefix;
        }
    }

    fn finalize(&mut self) {
        assert!(self.current_key_count == 0 || self.previous_key == self.max_key);
        if self.current_key_count > 0
            && self.spline_points.last().copied().unwrap().0 != self.previous_key
        {
            self.add_key_to_spline(self.previous_key, self.previous_position as f64);
        }

        let num_spline_points = u32::try_from(self.spline_points.len())
            .expect("radix spline contains more spline points than u32 can index");
        for i in (self.previous_prefix + 1)..self.radix_table.len() {
            self.radix_table[i] = num_spline_points;
        }
    }

    pub fn build(mut self) -> RadixSpline<K> {
        self.finalize();

        // Avoid having a too large data structure
        self.spline_points.shrink_to_fit();
        self.radix_table.shrink_to_fit();

        RadixSpline {
            min_key: self.min_key,
            max_key: self.max_key,
            current_key_count: self.current_key_count,
            shift_bits: self.shift_bits,
            max_error: self.max_error,
            radix_table: self.radix_table,
            spline_points: self.spline_points,
        }
    }
}

fn num_shift_bits<K: Key>(diff: K, radix_bits: u64) -> usize {
    let significant_bits = u64::from(K::zero().count_zeros() - diff.leading_zeros());
    if significant_bits < radix_bits {
        0
    } else {
        usize::try_from(significant_bits - radix_bits).expect("shift count does not fit in usize")
    }
}

fn compute_orientation(p1: (f64, f64), p2: (f64, f64)) -> Orientation {
    let expr = p1.1.mul_add(p2.0, -(p2.1 * p1.0));
    if expr > f64::EPSILON {
        Orientation::Clockwise
    } else if expr < -f64::EPSILON {
        Orientation::AntiClockwise
    } else {
        Orientation::Colinear
    }
}

pub trait Key: num_traits::PrimInt + num_traits::Unsigned + std::fmt::Debug {
    fn to_f64(self) -> f64;
}

impl Key for u32 {
    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}

impl Key for u64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hegel::TestCase;
    use hegel::generators::{self as gs, Generator};

    #[test]
    fn basic_behavioural() {
        let mut builder = RadixSpline::builder(0u64, 1000u64);
        builder.add_keys(0..=1000);
        let spline = builder.build();

        let range = spline.find(50);

        assert!(range.contains(&50));
    }

    /// A spline uses the key's native width without changing lookup behaviour.
    #[test]
    fn supports_u32_keys() {
        let mut builder = RadixSpline::builder(0u32, 1000u32);
        builder.add_keys(0..=1000);
        let spline = builder.build();

        assert!(spline.find(50).contains(&50));
    }

    #[hegel::test]
    fn construct_spline(tc: TestCase) {
        let mut data = tc.draw(
            gs::vecs(gs::integers::<u64>())
                .min_size(2)
                .max_size(20)
                .filter(|v| {
                    let first = &v[0];
                    v[1..].iter().any(|x| x != first)
                }),
        );
        data.sort();

        let min = data[0];
        let max = data[data.len() - 1];

        let mut builder = RadixSpline::builder(min, max);
        builder
            .max_error(tc.draw(gs::integers().min_value(5).max_value(30)))
            .radix_bits(tc.draw(gs::integers().max_value(25).min_value(8)));
        builder.add_keys(data.iter().copied());
        let _spline = builder.build();
    }

    #[hegel::test]
    fn can_find_elements(tc: TestCase) {
        let mut data = tc.draw(gs::vecs(gs::integers::<u64>()).min_size(2).filter(|v| {
            let first = &v[0];
            v[1..].iter().any(|x| x != first)
        }));
        data.sort();

        let min = data[0];
        let max = data[data.len() - 1];

        let mut builder = RadixSpline::builder(min, max);
        builder.add_keys(data.iter().copied());
        let spline = builder.build();

        let search_for = tc.draw(gs::sampled_from(&data));

        let range = spline.find(search_for);
        let max_width = 2 * spline.max_error as usize + 2;
        assert!(range.len() <= max_width);
        for i in range {
            assert!(data[i] <= search_for);
            if data[i] == search_for {
                return;
            }
        }
        panic!("Didn't find element");
    }

    #[hegel::test]
    #[should_panic]
    fn data_must_be_sorted(tc: TestCase) {
        let data = tc.draw(
            gs::vecs(gs::integers::<u64>())
                .min_size(2)
                .filter(|v| {
                    let first = &v[0];
                    v[1..].iter().any(|x| x != first)
                })
                .filter(|v| {
                    let mut x = v.clone();
                    x.sort();

                    v != &x
                }),
        );

        let min = data.iter().copied().min().unwrap();
        let max = data.iter().copied().max().unwrap();

        let mut builder = RadixSpline::builder(min, max);
        builder.add_keys(data.iter().copied());
    }

    #[cfg(kani)]
    #[kani::proof]
    #[kani::unwind(16)]
    fn spline_searching_checks() {
        let data = vec![
            1, 2, 3, 55, 56, 57, 110, 111, 112, 165, 166, 167, 277, 278, 279,
        ];
        let mut builder = RadixSpline::builder(1u64, 279u64);
        builder.max_error(3);
        builder.radix_bits(4);
        builder.add_keys(data.iter().copied());
        let spline = builder.build();

        let max_width = 2 * spline.max_error as usize + 2;

        let selected = kani::any::<u8>() as usize;
        kani::assume(selected < data.len());

        let search_for = data[selected];
        let range = spline.find(search_for);
        assert!(range.len() <= max_width);

        let mut found = false;
        for index in range {
            if data[index] == search_for {
                found = true;
            }
        }
        assert!(found);

        let arbitrary_key = u64::from(kani::any::<u16>());
        kani::assume(arbitrary_key <= 280);

        let range = spline.find(arbitrary_key);

        assert!(range.len() <= max_width);
        assert!(range.start <= range.end);
        assert!(range.end <= data.len());
    }

    #[cfg(kani)]
    #[kani::proof]
    fn shift_bits_valid() {
        let diff: u64 = kani::any();
        let radix_bits: u64 = kani::any();
        kani::assume(diff > 0);
        kani::assume(radix_bits > 0);
        kani::assume(radix_bits <= 64);

        num_shift_bits(diff, radix_bits);
    }

    #[cfg(kani)]
    #[kani::proof]
    fn compute_orientation_numerically_safe() {
        let x1: u64 = kani::any();
        let y1: f64 = kani::any();
        let x2: u64 = kani::any();
        let y2: f64 = kani::any();

        kani::assume(y1 >= 0.0 && y1 < u64::MAX as f64);
        kani::assume(y2 >= 0.0 && y2 < u64::MAX as f64);

        compute_orientation((x1 as f64, y1), (x2 as f64, y2));
    }
}
