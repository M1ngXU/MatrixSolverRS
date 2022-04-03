use std::fmt::Display;

use crate::{fraction::Fraction, row::Row};

#[derive(Debug, Clone)]
pub struct Matrix {
	rows: Vec<Row>,
	state: MatrixState,
	row_sequence: Vec<usize>,
}
impl Matrix {
	pub fn new_with_rows(rows: Vec<Row>) -> Self {
		rows.iter()
			.enumerate()
			.map(|(i, r)| (i, r.left().len()))
			.for_each(|(i, e)| {
				if e < rows.len() {
					panic!(
						"Row {} needs at least {} elements on the left side.",
						i + 1,
						rows.len()
					)
				} else {
					()
				}
			});
		let starting_state = if rows.len() == 1 {
			MatrixState::NormalizeRow(0)
		} else {
			MatrixState::Null(0)
		};
		Self::new_with_state(rows, starting_state)
	}

	pub fn new_with_state(rows: Vec<Row>, state: MatrixState) -> Self {
		let row_sequence = (0..rows.len()).into_iter().collect::<Vec<usize>>();
		Self::new(rows, state, row_sequence)
	}

	pub fn new(rows: Vec<Row>, state: MatrixState, row_sequence: Vec<usize>) -> Self {
		Self {
			state,
			row_sequence,
			rows,
		}
	}

	pub fn rows(&self) -> &Vec<Row> {
		&self.rows
	}

	pub fn state(&self) -> &MatrixState {
		&self.state
	}

	fn null_row(&self, row: usize) -> Matrix {
		let first_row = self.rows[self.row_sequence[row]].clone();
		Matrix::new(
			self.rows
				.iter()
				.enumerate()
				.map(|(i, r)| {
					if self.row_sequence.iter().position(|n| n == &i).unwrap()
						<= row
					{
						r.clone()
					} else {
						r.clone() * first_row[row as isize] - first_row.clone() * r[row as isize]
					}
				})
				.collect::<Vec<Row>>(),
			if row + 2 == self.row_sequence.len() {
				MatrixState::NormalizeRow(row + 1)
			} else {
				MatrixState::Null(row + 1)
			},
			self.row_sequence.clone(),
		)
	}

	fn normalize_row(&self, row: usize) -> Option<Matrix> {
		let mut new = self.clone();
		let factor = new.rows[self.row_sequence[row]][row as isize];
		new.rows[self.row_sequence[row]] /= factor;
		new.state = if self.row_sequence[row] == 0 {
			MatrixState::Done
		} else {
			MatrixState::ReInsert(row - 1)
		};
		if new
			.rows
			.iter()
			.any(|r| r.left().iter().all(|f| f.sign() == 0))
		{
			None
		} else {
			Some(new)
		}
	}

	fn reinsert_row(&self, row: usize) -> Matrix {
		// assuming that the rows n + 1..
		// are zeroed with only one `1` at `n`
		//
		// something like
		// 0 1 0 | 0
		// 0 0 1 | 2
		// with n = 0
		let mut new = self.clone();
		for i in row..self.rows.len() - 1 {
			new.rows[self.row_sequence[row]] = new.rows[self.row_sequence[row]].clone() * new.rows[self.row_sequence[i + 1]][i as isize + 1] // should be 1 though
                    		- new.rows[self.row_sequence[i + 1]].clone() * new.rows[self.row_sequence[row]][i as isize + 1];
		}
		new.state = MatrixState::NormalizeRow(row);
		new
	}

	pub fn calculate_next(&self) -> Option<Matrix> {
		match self.state {
			MatrixState::Null(s) => Some(self.null_row(s)),
			MatrixState::NormalizeRow(n) => self.normalize_row(n),
			MatrixState::ReInsert(n) => Some(self.reinsert_row(n)),
			MatrixState::Done => None,
		}
	}
}
impl PartialEq for Matrix {
	fn eq(&self, other: &Self) -> bool {
		self.rows == other.rows && self.state == other.state
	}
}
impl Display for Matrix {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let longest = self
			.rows
			.iter()
			.filter_map(|r| {
				r.left()
					.iter()
					.chain(r.right())
					.map(|f| f.to_string().len())
					.max()
			})
			.max()
			.unwrap_or(0);
		f.pad(
			self.rows
				.iter()
				.map(|r| {
					format!(
						"({} | {})",
						pad_row(r.left(), longest, " "),
						pad_row(r.right(), longest, " ")
					)
				})
				.collect::<Vec<String>>()
				.join("\n")
				.as_str(),
		)
	}
}
fn pad_row(v: &Vec<Fraction>, l: usize, s: &str) -> String {
	v.iter()
		.map(|f| format!("{f:>l$}"))
		.collect::<Vec<String>>()
		.join(s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixState {
	Null(usize),
	NormalizeRow(usize),
	ReInsert(usize),
	Done,
}
impl Display for MatrixState {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.pad(&match self {
			MatrixState::Null(n) => format!("Nulling first {} col(s)", n + 1),
			MatrixState::NormalizeRow(n) => format!("Normalizing row {}", n + 1),
			MatrixState::ReInsert(n) => format!("Using nulled rows to reinsert row {}", n + 1),
			MatrixState::Done => format!("Done"),
		})
	}
}

#[cfg(test)]
mod test {
	use crate::{fraction::Fraction, matrix::MatrixState, row::Row};

	use super::Matrix;

	type F = Fraction;

	#[test]
	fn get_next_matrix() {
		let step0 = Matrix::new(
			vec![
				Row::new(vec![1.into(), 1.into(), 2.into()], vec![8.into()]),
				Row::new(vec![3.into(), 8.into(), 9.into()], vec![3.into()]),
				Row::new(vec![4.into(), 2.into(), 3.into()], vec![1.into()]),
			],
			MatrixState::Null(0),
			vec![0, 2, 1],
		);
		let step1 = step0.calculate_next().unwrap();
		assert_eq!(
			step1,
			Matrix::new_with_state(
				vec![
					Row::new(vec![1.into(), 1.into(), 2.into()], vec![8.into()]),
					Row::new(
						vec![F::ZERO, 5.into(), 3.into()],
						vec![F::negative_n(21)]
					),
					Row::new(vec![F::ZERO, (-2).into(), (-5).into()], vec![F::negative_n(31)]),
				],
				MatrixState::Null(1)
			)
		);
		let step2 = step1.calculate_next().unwrap();
		assert_eq!(
			step2,
			Matrix::new_with_state(
				vec![
					Row::new(vec![1.into(), 1.into(), 2.into()], vec![8.into()]),
					Row::new(vec![F::ZERO, F::ZERO, 19.into()], vec![197.into()]),
					Row::new(vec![F::ZERO, (-2).into(), (-5).into()], vec![F::negative_n(31)]),
				],
				MatrixState::NormalizeRow(2)
			)
		);
		let step3 = step2.calculate_next().unwrap();
		assert_eq!(
			step3,
			Matrix::new_with_state(
				vec![
					Row::new(vec![1.into(), 1.into(), 2.into()], vec![8.into()]),
					Row::new(
						vec![F::ZERO, F::ZERO, 1.into()],
						vec![F::positive_n(197) / 19.into()]
					),
					Row::new(
						vec![F::ZERO, F::negative_n(2), F::negative_n(5)],
						vec![F::negative_n(31)]
					),
				],
				MatrixState::ReInsert(1)
			)
		);
		let step4 = step3.calculate_next().unwrap();
		assert_eq!(
			step4,
			Matrix::new_with_state(
				vec![
					Row::new(vec![1.into(), 1.into(), 2.into()], vec![8.into()]),
					Row::new(
						vec![F::ZERO, F::ZERO, 1.into()],
						vec![F::positive_n(197) / 19.into()]
					),
					Row::new(
						vec![F::ZERO, F::negative_n(2), F::ZERO],
						vec![F::negative_n(31) + F::positive_n(985) / 19.into()]
					),
				],
				MatrixState::NormalizeRow(1)
			)
		);
		let step5 = step4.calculate_next().unwrap();
		assert_eq!(
			step5,
			Matrix::new_with_state(
				vec![
					Row::new(vec![1.into(), 1.into(), 2.into()], vec![8.into()]),
					Row::new(
						vec![F::ZERO, F::ZERO, 1.into()],
						vec![F::positive_n(197) / 19.into()]
					),
					Row::new(
						vec![F::ZERO, 1.into(), F::ZERO],
						vec![
							(F::negative_n(31) + F::positive_n(985) / 19.into()) / F::negative_n(2)
						]
					),
				],
				MatrixState::ReInsert(0)
			)
		);
		assert_eq!(
			step5.calculate_next().unwrap().calculate_next().unwrap(),
			Matrix::new_with_state(
				vec![
					Row::new(
						vec![1.into(), F::ZERO, F::ZERO],
						vec![
							F::positive_n(8)
								- (F::negative_n(31) + F::positive_n(985) / 19.into())
									/ F::negative_n(2) - F::positive_n(197) / 19.into() * 2.into()
						]
					),
					Row::new(
						vec![F::ZERO, F::ZERO, 1.into()],
						vec![F::positive_n(197) / 19.into()]
					),
					Row::new(
						vec![F::ZERO, 1.into(), F::ZERO],
						vec![
							(F::negative_n(31) + F::positive_n(985) / 19.into()) / F::negative_n(2)
						]
					),
				],
				MatrixState::Done
			)
		);
	}
}
