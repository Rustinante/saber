use std::cmp::{max, min};
use std::ops::{Sub, SubAssign};

use num::integer::Integer;
use num::traits::cast::ToPrimitive;

use crate::interval::traits::{CoalesceIntervals, Interval};
use crate::set::ordered_integer_set::{ContiguousIntegerSet, OrderedIntegerSet};
use crate::set::traits::Set;

impl<E: Integer + Copy + ToPrimitive> Sub<&ContiguousIntegerSet<E>> for ContiguousIntegerSet<E> {
    type Output = OrderedIntegerSet<E>;
    fn sub(self, rhs: &ContiguousIntegerSet<E>) -> Self::Output {
        let a = self.get_start();
        let b = self.get_end();
        let c = rhs.get_start();
        let d = rhs.get_end();
        if self.is_empty() || rhs.is_empty() {
            return OrderedIntegerSet::from(vec![self]);
        }
        // [a, b] - [c, d]
        let set = OrderedIntegerSet::from(vec![
            ContiguousIntegerSet::new(a, min(b, c - E::one())),
            ContiguousIntegerSet::new(max(d + E::one(), a), b),
        ]);
        set.into_non_empty_intervals()
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub for ContiguousIntegerSet<E> {
    type Output = OrderedIntegerSet<E>;

    #[inline]
    fn sub(self, rhs: ContiguousIntegerSet<E>) -> Self::Output {
        self - &rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub<&ContiguousIntegerSet<E>> for OrderedIntegerSet<E> {
    type Output = Self;
    fn sub(self, rhs: &ContiguousIntegerSet<E>) -> Self::Output {
        let diff_intervals: Vec<ContiguousIntegerSet<E>> = self.intervals.iter()
                                                               .flat_map(|i| (*i - rhs).intervals)
                                                               .collect();
        OrderedIntegerSet::from(diff_intervals)
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub<ContiguousIntegerSet<E>> for OrderedIntegerSet<E> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: ContiguousIntegerSet<E>) -> Self::Output {
        self - &rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> SubAssign<&ContiguousIntegerSet<E>> for OrderedIntegerSet<E> {
    fn sub_assign(&mut self, rhs: &ContiguousIntegerSet<E>) {
        *self = self.to_owned() - rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> SubAssign<ContiguousIntegerSet<E>> for OrderedIntegerSet<E> {
    #[inline]
    fn sub_assign(&mut self, rhs: ContiguousIntegerSet<E>) {
        *self = self.to_owned() - &rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub<&OrderedIntegerSet<E>> for ContiguousIntegerSet<E> {
    type Output = OrderedIntegerSet<E>;
    fn sub(self, rhs: &OrderedIntegerSet<E>) -> Self::Output {
        let mut diff = OrderedIntegerSet::from(vec![self]);
        for interval in rhs.intervals_iter() {
            diff -= interval;
        }
        diff.into_coalesced()
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub<OrderedIntegerSet<E>> for ContiguousIntegerSet<E> {
    type Output = OrderedIntegerSet<E>;

    #[inline]
    fn sub(self, rhs: OrderedIntegerSet<E>) -> Self::Output {
        self - &rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub<&OrderedIntegerSet<E>> for OrderedIntegerSet<E> {
    type Output = Self;
    fn sub(self, rhs: &OrderedIntegerSet<E>) -> Self::Output {
        let mut diff = Vec::new();
        let mut rhs_i = 0;
        let num_rhs_intervals = rhs.intervals.len();
        for interval in self.intervals.iter() {
            let mut d = OrderedIntegerSet::from_contiguous_integer_sets(vec![*interval]);
            while rhs_i < num_rhs_intervals && rhs.intervals[rhs_i].end < interval.start {
                rhs_i += 1;
            }
            while rhs_i < num_rhs_intervals && rhs.intervals[rhs_i].end <= interval.end {
                d -= rhs.intervals[rhs_i];
                rhs_i += 1;
            }
            if rhs_i < num_rhs_intervals && rhs.intervals[rhs_i].start <= interval.end {
                d -= rhs.intervals[rhs_i];
            }
            for i in d.intervals.into_iter() {
                diff.push(i);
            }
        }
        OrderedIntegerSet::from_contiguous_integer_sets(diff)
    }
}

impl<E: Integer + Copy + ToPrimitive> Sub<OrderedIntegerSet<E>> for OrderedIntegerSet<E> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: OrderedIntegerSet<E>) -> Self::Output {
        self - &rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> SubAssign<&OrderedIntegerSet<E>> for OrderedIntegerSet<E> {
    fn sub_assign(&mut self, rhs: &OrderedIntegerSet<E>) {
        *self = self.to_owned() - rhs
    }
}

impl<E: Integer + Copy + ToPrimitive> SubAssign<OrderedIntegerSet<E>> for OrderedIntegerSet<E> {
    #[inline]
    fn sub_assign(&mut self, rhs: OrderedIntegerSet<E>) {
        *self = self.to_owned() - &rhs
    }
}
