use std::ops::{DivAssign, Index, IndexMut, Mul, MulAssign, Sub};

use crate::fraction::Fraction;

#[derive(Debug, PartialEq, Clone)]
pub struct Row {
	left: Vec<Fraction>,
	right: Vec<Fraction>,
}
impl Row {
	pub fn new(left: Vec<Fraction>, right: Vec<Fraction>) -> Self {
		Self { left, right }
	}

	pub fn left(&self) -> &Vec<Fraction> {
		&self.left
	}

	pub fn right(&self) -> &Vec<Fraction> {
		&self.right
	}
}
impl MulAssign<Fraction> for Row {
	fn mul_assign(&mut self, rhs: Fraction) {
		self.left.iter_mut().for_each(|n| *n *= rhs);
		self.right.iter_mut().for_each(|n| *n *= rhs);
	}
}
impl DivAssign<Fraction> for Row {
	#[allow(clippy::suspicious_op_assign_impl)]
	fn div_assign(&mut self, rhs: Fraction) {
		*self *= rhs.swapped();
	}
}
impl Mul<Fraction> for Row {
	type Output = Row;

	fn mul(self, rhs: Fraction) -> Self::Output {
		let mut new = self;
		new *= rhs;
		new
	}
}
impl Sub<Row> for Row {
	type Output = Row;

	fn sub(self, rhs: Row) -> Self::Output {
		let mut new = self;
		new.left
			.iter_mut()
			.enumerate()
			.for_each(|(i, n)| *n -= rhs.left[i]);
		new.right
			.iter_mut()
			.enumerate()
			.for_each(|(i, n)| *n -= rhs.right[i]);
		new
	}
}
impl Index<isize> for Row {
	type Output = Fraction;

	fn index(&self, index: isize) -> &Self::Output {
		if index < 0 {
			&self.right[(-1 - index) as usize]
		} else {
			&self.left[index as usize]
		}
	}
}
impl IndexMut<isize> for Row {
	fn index_mut(&mut self, index: isize) -> &mut Self::Output {
		if index < 0 {
			&mut self.right[(-1 - index) as usize]
		} else {
			&mut self.left[index as usize]
		}
	}
}

#[cfg(test)]
mod test {
	use crate::fraction::Fraction;

	use super::Row;
	type F = Fraction;

	#[test]
	fn mul() {
		assert_eq!(
			Row::new(vec![F::ZERO, 1.into(), 2.into()], vec![3.into(), 4.into()]) * 2.into(),
			Row::new(vec![0.into(), 2.into(), 4.into()], vec![6.into(), 8.into()])
		);
	}

	#[test]
	fn sub() {
		assert_eq!(
			Row::new(vec![0.into(), 1.into(), 2.into()], vec![3.into(), 4.into()])
				- Row::new(vec![0.into(), 1.into(), 0.into()], vec![0.into(), 1.into()]),
			Row::new(vec![0.into(), 0.into(), 2.into()], vec![3.into(), 3.into()])
		);
	}

	#[test]
	fn index() {
		let cut = Row::new(vec![0.into(), 1.into(), 2.into()], vec![3.into(), 4.into()]);
		assert_eq!(cut[0], 0.into());
		assert_eq!(cut[1], 1.into());
		assert_eq!(cut[2], 2.into());
		assert_eq!(cut[-1], 3.into());
		assert_eq!(cut[-2], 4.into());
	}
}
