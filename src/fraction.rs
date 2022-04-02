use std::{
	cmp::Ordering,
	fmt::{Debug, Display},
	ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

const fn gcd(mut a: usize, mut b: usize) -> usize {
	while b != 0 {
		let t = b;
		b = a % b;
		a = t;
	}
	a
}

#[derive(Clone, Copy)]
pub struct Fraction {
	negative: bool,
	numerator: usize,
	denominator: usize,
}
impl Fraction {
	pub const ZERO: Fraction = Self::positive_n(0);

	pub const fn new(negative: bool, numerator: usize, denominator: usize) -> Self {
		let gcd = gcd(numerator, denominator);
		Self {
			denominator: denominator / gcd,
			negative: negative && numerator > 0,
			numerator: numerator / gcd,
		}
	}

	pub const fn positive(numerator: usize, denominator: usize) -> Self {
		Self::new(false, numerator, denominator)
	}

	pub const fn negative(numerator: usize, denominator: usize) -> Self {
		Self::new(true, numerator, denominator)
	}

	pub const fn positive_n(n: usize) -> Self {
		Self::positive(n, 1)
	}

	pub const fn negative_n(n: usize) -> Self {
		Self::negative(n, 1)
	}

	pub const fn sign(&self) -> isize {
		if self.numerator == 0 {
			0
		} else if self.negative {
			-1
		} else {
			1
		}
	}

	pub fn swap(&mut self) {
		if self.sign() != 0 {
			std::mem::swap(&mut self.numerator, &mut self.denominator);
		}
	}

	pub fn swapped(&self) -> Self {
		let mut new = self.clone();
		new.swap();
		new
	}

	pub fn reduce(&mut self) {
		if self.numerator == 0 {
			self.denominator = 1;
			self.negative = false;
		} else {
			let gcd = gcd(self.numerator, self.denominator);
			self.numerator /= gcd;
			self.denominator /= gcd;
		}
	}

	pub fn reduced(&self) -> Self {
		Self::new(self.negative, self.numerator, self.denominator)
	}

	pub fn as_f64(&self) -> f64 {
		self.sign() as f64 * self.numerator as f64 / self.denominator as f64
	}

    pub fn abs(&self) -> Self {
        Self::positive(self.numerator, self.denominator)
    }

    pub fn is_negative(&self) -> bool {
        self.negative
    }
}
impl From<isize> for Fraction {
	fn from(n: isize) -> Self {
		match n.partial_cmp(&0) {
			Some(Ordering::Greater) => Self::positive_n(n as usize),
			Some(Ordering::Equal) => Self::ZERO,
			Some(Ordering::Less) => Self::negative_n(n.abs() as usize),
			None => todo!(),
		}
	}
}
impl Add<Fraction> for Fraction {
	type Output = Fraction;

	fn add(self, rhs: Fraction) -> Self::Output {
		let mut new = self.clone();
		new += rhs;
		new
	}
}
impl AddAssign<Fraction> for Fraction {
	fn add_assign(&mut self, rhs: Fraction) {
		let new_num = self.numerator as isize * rhs.denominator as isize * self.sign()
			+ rhs.numerator as isize * self.denominator as isize * rhs.sign();
		self.negative = new_num < 0;
		self.numerator = new_num.abs() as usize;
		self.denominator *= rhs.denominator;
		self.reduce();
	}
}
impl Sub<Fraction> for Fraction {
	type Output = Fraction;

	fn sub(self, rhs: Fraction) -> Self::Output {
		self + -rhs
	}
}
impl SubAssign<Fraction> for Fraction {
	fn sub_assign(&mut self, rhs: Fraction) {
		*self += -rhs;
	}
}
impl Mul<Fraction> for Fraction {
	type Output = Fraction;

	fn mul(self, rhs: Fraction) -> Self::Output {
		let mut new = self.clone();
		new *= rhs;
		new
	}
}
impl MulAssign<Fraction> for Fraction {
	fn mul_assign(&mut self, rhs: Fraction) {
		self.negative ^= rhs.negative;
		self.numerator *= rhs.numerator;
		self.denominator *= rhs.denominator;
		self.reduce();
	}
}
impl Div<Fraction> for Fraction {
	type Output = Fraction;

	fn div(self, rhs: Fraction) -> Self::Output {
		self * rhs.swapped()
	}
}
impl DivAssign<Fraction> for Fraction {
	fn div_assign(&mut self, rhs: Fraction) {
		*self *= rhs.swapped();
	}
}
impl Neg for Fraction {
	type Output = Fraction;

	fn neg(self) -> Self::Output {
		Self::new(!self.negative, self.numerator, self.denominator)
	}
}
impl PartialOrd for Fraction {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.as_f64().partial_cmp(&other.as_f64())
	}
}
impl PartialEq for Fraction {
	fn eq(&self, other: &Self) -> bool {
		let a = self.reduced();
		let b = other.reduced();
		a.negative == b.negative && a.numerator == b.numerator && a.denominator == b.denominator
	}
}
impl Display for Fraction {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.pad(&format!(
			"{}{}{}",
			if self.negative {
				"-"
			} else if f.sign_plus() {
				"+"
			} else {
				""
			},
			self.numerator,
			if self.denominator == 1 {
				String::new()
			} else {
				format!("/{}", self.denominator)
			}
		))
	}
}
impl Debug for Fraction {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}

#[cfg(test)]
mod test {
	use crate::fraction::Fraction;

	#[test]
	fn truncate() {
		assert_eq!(
			Fraction::new(true, 7, 14).reduced(),
			Fraction::new(true, 1, 2)
		);
	}

	#[test]
	fn negate() {
		assert_eq!(-Fraction::new(false, 1, 2), Fraction::new(true, 1, 2));
	}

	#[test]
	fn add() {
		assert_eq!(
			Fraction::new(false, 1, 2) + Fraction::new(false, 1, 2),
			Fraction::new(false, 1, 1)
		);
		assert_eq!(
			(Fraction::new(false, 1, 2) + Fraction::new(true, 1, 2)).sign(),
			0
		);
	}

	#[test]
	fn mul() {
		assert_eq!(
			Fraction::new(false, 3, 2) * Fraction::new(true, 1, 3),
			Fraction::new(true, 1, 2)
		);
	}

	#[test]
	fn div() {
		assert_eq!(
			Fraction::new(false, 3, 2) / Fraction::new(false, 3, 1),
			Fraction::new(false, 1, 2)
		);
	}
}
