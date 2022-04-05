use std::fmt::{Display, Debug};

use crate::{fraction::Fraction, row::Row};

fn recursive_determine_best(cols: Vec<usize>, rows: &[Row], max: usize) -> (Vec<usize>, usize) {
	if cols.len() == max {
		return (cols, 0);
	}
	let mut best = (0..max)
		.into_iter()
		.filter(|i| cols.iter().all(|c| c != i))
		.map(|i| {
			(
				i,
				rows.iter()
					.filter(|r| cols.iter().all(|i| r.left()[*i] == Fraction::ZERO))
					.filter(|r| r.left()[i] == Fraction::ZERO)
					.count(),
			)
		})
		.collect::<Vec<(usize, usize)>>();
	best.sort_by(|a, b| a.1.cmp(&b.1).reverse());
	best.iter()
		.rev()
		// using skip_while since it's sorted
		.skip_while(|i| i.1 != best[0].1)
		.map(|(i, v)| {
			let mut new_cols = cols.clone();
			new_cols.push(*i);
			let (new_cols, nulls) = recursive_determine_best(new_cols, rows, max);
			(new_cols, nulls + *v)
		})
		.max_by_key(|(_, i)| *i)
		.unwrap()
}

#[derive(Clone)]
pub struct Matrix {
	rows: Vec<Row>,
	state: MatrixState,
	row_sequence: Vec<usize>,
	col_sequence: Vec<usize>,
}
impl Matrix {
	pub fn create<const N: usize, const M: usize>(
		left: [[Fraction; N]; N],
		right: [[Fraction; M]; N],
	) -> Self {
		let rows = left
			.into_iter()
			.zip(right.into_iter())
			.map(|(l, r)| Row::new(l.to_vec(), r.to_vec()))
			.collect::<Vec<Row>>();
		let mut new = Self::new_with_rows(rows);
		new.optimize_indeces();
		new
	}

	pub fn optimize_indeces(&mut self) {
		if let Some(max) = self.rows.iter().map(|r| r.left().len()).max() {
			let cols = recursive_determine_best(Vec::new(), &self.rows, max).0;
			let mut rows = self
				.rows
				.iter()
				.map(|r| {
					cols.iter()
						.take_while(|i| {
							r.left().get(**i).unwrap_or(&Fraction::positive_n(1)) == &Fraction::ZERO
						})
						.count()
				})
				.enumerate()
				.collect::<Vec<(usize, usize)>>();
			rows.sort_unstable_by_key(|(_, i)| *i);
			self.col_sequence = cols;
			self.row_sequence = rows.iter().map(|(i, _)| *i).collect::<Vec<usize>>();
		} else {
			self.col_sequence = Vec::new();
			self.row_sequence = Vec::new();
		}
	}

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
		Self::new(rows, state, row_sequence.clone(), row_sequence)
	}

	pub fn new(
		rows: Vec<Row>,
		state: MatrixState,
		row_sequence: Vec<usize>,
		col_sequence: Vec<usize>,
	) -> Self {
		Self {
			state,
			col_sequence,
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

	fn null_row(&self, index: usize) -> Option<Matrix> {
		let relevant_cell = self.rows[self.row_sequence[index]][self.col_sequence[index] as isize];
		if relevant_cell == Fraction::ZERO {
			return None;
		}
		let mut r = Matrix::new(
			self.rows
				.iter()
				.enumerate()
				.map(|(i, r)| {
					if self.row_sequence.iter().position(|n| n == &i).unwrap() <= index {
						r.clone()
					} else {
						r.clone() * relevant_cell
							- self.rows[self.row_sequence[index]].clone()
								* r[self.col_sequence[index] as isize]
					}
				})
				.collect::<Vec<Row>>(),
			if index + 2 == self.row_sequence.len() {
				MatrixState::NormalizeRow(index + 1)
			} else {
				MatrixState::Null(index + 1)
			},
			self.row_sequence.clone(),
			self.col_sequence.clone(),
		);
		r.optimize_indeces();
		r.update_state();
		Some(r)
	}

	fn normalize_row(&self, index: usize) -> Option<Matrix> {
		let mut new = self.clone();
		let factor = new.rows[self.row_sequence[index]][self.col_sequence[index] as isize];
		new.rows[self.row_sequence[index]] /= factor;
		if new
			.rows
			.iter()
			.any(|r| r.left().iter().all(|f| f == &Fraction::ZERO))
		{
			None
		} else {
			new.update_state();
			Some(new)
		}
	}

	fn reinsert_row(&self, index: usize) -> Matrix {
		// assuming that the rows n + 1..
		// are zeroed with only one `1` at `n`
		//
		// something like
		// 0 1 0 | 0
		// 0 0 1 | 2
		// with n = 0
		let mut new = self.clone();
		for i in index..self.rows.len() - 1 {
			new.rows[self.row_sequence[index]] = new.rows[self.row_sequence[index]].clone() * new.rows[self.row_sequence[i + 1]][self.col_sequence[i + 1] as isize] // should be 1 though
				- new.rows[self.row_sequence[i + 1]].clone() * new.rows[self.row_sequence[index]][self.col_sequence[i + 1] as isize];
		}
		new.update_state();
		new
	}

	pub fn update_state(&mut self) {
		match self.state {
			MatrixState::Initial => {
				self.state = MatrixState::Null(0);
				self.update_state();
			}
			MatrixState::Null(_) => {
				if let Some(n) = self
					.row_sequence
					.iter()
					.enumerate()
					.map(|(n, i)| (n, &self.rows[*i]))
					.position(|(n, r)| {
						self.col_sequence[..n]
							.iter()
							.any(|i| r.left()[*i] != Fraction::ZERO)
					}) {
					self.state = MatrixState::Null(n - 1);
				} else {
					self.state = MatrixState::NormalizeRow(self.rows.len() - 1);
					self.update_state()
				}
			}
			MatrixState::NormalizeRow(_) | MatrixState::ReInsertRow(_) => {
				self.state = self
					.row_sequence
					.iter()
					.enumerate()
					.map(|(n, i)| (n, &self.rows[*i]))
					.rev()
					.find_map(|(n, r)| {
						if self.col_sequence[n + 1..]
							.iter()
							.any(|i| r.left()[*i] != Fraction::ZERO)
						{
							Some(MatrixState::ReInsertRow(n))
						} else if r.left()[self.col_sequence[n]] != Fraction::ONE {
							Some(MatrixState::NormalizeRow(n))
						} else {
							None
						}
					})
					.unwrap_or(MatrixState::Done);
			}
			MatrixState::Done => {}
		}
	}

	pub fn calculate_next(&self) -> Option<Matrix> {
		match self.state {
			MatrixState::Initial => panic!("Update state first!"),
			MatrixState::Null(s) => self.null_row(s),
			MatrixState::NormalizeRow(n) => self.normalize_row(n),
			MatrixState::ReInsertRow(n) => Some(self.reinsert_row(n)),
			MatrixState::Done => None,
		}
	}

	pub fn row_sequence(&self) -> &Vec<usize> {
		&self.row_sequence
	}

	pub fn col_sequence(&self) -> &Vec<usize> {
		&self.col_sequence
	}
}
impl PartialEq for Matrix {
	fn eq(&self, other: &Self) -> bool {
		self.rows == other.rows
	}
}
impl Debug for Matrix {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "\n{}", self)
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
fn pad_row(v: &[Fraction], l: usize, s: &str) -> String {
	v.iter()
		.map(|f| format!("{f:>l$}"))
		.collect::<Vec<String>>()
		.join(s)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixState {
	Initial,
	Null(usize),
	NormalizeRow(usize),
	ReInsertRow(usize),
	Done,
}
impl Display for MatrixState {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.pad(&match self {
			MatrixState::Initial => "Initial state".to_string(),
			MatrixState::Null(n) => format!("Nulling first {} col(s)", n + 1),
			MatrixState::NormalizeRow(n) => format!("Normalizing row {}", n + 1),
			MatrixState::ReInsertRow(n) => format!("Using nulled rows to reinsert row {}", n + 1),
			MatrixState::Done => "Done".to_string(),
		})
	}
}

/// A macro to easily create matrices.
/// The amount of columns on the left side must match the amount of rows.
///
/// # Example
/// ```
/// # use matrix_solver::matrix::Matrix;
/// # use matrix_solver::row::Row;
/// # use matrix_solver::fraction::Fraction;
/// # use matrix_solver::matrix;
/// let a = 99i32;
/// assert_eq!(
///     matrix![[1; a] | [(3, 4)], [2; 3] | [-1]],
///     Matrix::new_with_rows(vec![
///         Row::new(
///             vec![Fraction::from(1i32), Fraction::from(a)],
///             vec![Fraction::from((3i32, 4i32))]
///         ),
///         Row::new(
///             vec![Fraction::from(2i32), Fraction::from(3i32)],
///             vec![Fraction::from(-1i32)]
///         ),
///     ])
/// );
/// ```
#[macro_export]
macro_rules! matrix {
    ($([$($left:expr);+] | [$($right:expr);+]),+ $(,)?) => {
		$crate::matrix::Matrix::create([ $([$($left.into()),+]),+ ], [ $([$($right.into()),+]),+ ])
    };
}

#[cfg(test)]
mod test {
	use crate::{matrix::MatrixState, solve, solve_with_history};

	#[cfg(test)]
	mod optimize {
		#[test]
		fn leave_if_perfect() {
			let mut cut = matrix![[1; 1] | [1], [0; 1] | [1]];
			cut.optimize_indeces();
			assert_eq!(cut.row_sequence, vec![0, 1]);
			assert_eq!(cut.col_sequence, vec![0, 1]);
		}

		#[test]
		fn switch_cols() {
			let mut cut = matrix![[1; 1] | [1], [1; 0] | [1]];
			cut.optimize_indeces();
			assert_eq!(cut.row_sequence, vec![0, 1]);
			assert_eq!(cut.col_sequence, vec![1, 0]);
		}

		#[test]
		fn switch_rows() {
			let mut cut = matrix![[0; 1] | [1], [1; 1] | [1]];
			cut.optimize_indeces();
			assert_eq!(cut.row_sequence, vec![1, 0]);
			assert_eq!(cut.col_sequence, vec![0, 1]);
		}

		#[test]
		fn complex() {
			let mut cut = matrix![
				[0;0;1;1]|[1],
				[1;0;1;0]|[1],
				[1;1;0;1]|[1],
				[1;0;0;0]|[1]
			];
			cut.optimize_indeces();
			assert_eq!(cut.col_sequence, vec![1, 3, 2, 0]);
			assert_eq!(cut.row_sequence, vec![2, 0, 1, 3]);

			let mut cut = matrix![
				[0;1;1;1]|[1],
				[1;0;1;1]|[1],
				[1;0;0;1]|[1],
				[0;1;1;1]|[1]
			];
			cut.optimize_indeces();
			assert_eq!(cut.col_sequence[..2], [1, 2]);
		}
	}

	#[test]
	fn create_matrix() {
		let cut = matrix![
			[1 ; 0 ; 1] | [1],
			[0 ; 1 ; 1] | [1],
			[1 ; 0 ; 0] | [1]
		];
		assert_eq!(cut.col_sequence, vec![1, 2, 0]);
		assert_eq!(cut.row_sequence, vec![1, 0, 2]);

		let cut = matrix![
			[2; 1; 7; -2] | [5],
			[0; 0; 2; 1] | [3],
			[2; 2; 1; 0] | [1],
			[0; 0; 1; 1] | [2],
		];
		assert_eq!(cut.col_sequence, vec![0, 1, 2, 3]);
		assert_eq!(cut.row_sequence, vec![0, 2, 1, 3]);
	}

	#[test]
	fn get_next_matrix() {
		let initial = matrix![
			[0 ; 1 ; 0 ; -1] | [ 1],
			[1 ; 1 ; 4 ;  2] | [ 3],
			[0 ; 2 ; 1 ;  1] | [ 5],
			[1 ; 0 ; 1 ;  0] | [-1]
		];
		assert_eq!(initial.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(initial.row_sequence, vec![1, 3, 2, 0]);
		assert_eq!(initial.state, MatrixState::Null(0));

		let step1 = initial.calculate_next().unwrap();
		assert_eq!(
			step1,
			matrix![
				[0 ;  1 ;  0 ; -1] | [ 1],
				[1 ;  1 ;  4 ;  2] | [ 3],
				[0 ;  2 ;  1 ;  1] | [ 5],
				[0 ; -1 ; -3 ; -2] | [-4]
			]
		);
		assert_eq!(step1.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step1.row_sequence, vec![1, 2, 3, 0]);
		assert_eq!(step1.state, MatrixState::Null(1));

		let step2 = step1.calculate_next().unwrap();
		assert_eq!(
			step2,
			matrix![
				[0 ; 1 ; 0 ; -1] | [ 1],
				[1 ; 1 ; 4 ;  2] | [ 3],
				[0 ; 2 ; 1 ;  1] | [ 5],
				[0 ; 5 ; 0 ;  1] | [11]
			]
		);
		assert_eq!(step2.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step2.row_sequence, vec![1, 2, 0, 3]);
		assert_eq!(step2.state, MatrixState::Null(2));

		let step3 = step2.calculate_next().unwrap();
		assert_eq!(
			step3,
			matrix![
				[0 ; 1 ; 0 ; -1] | [1],
				[1 ; 1 ; 4 ;  2] | [3],
				[0 ; 2 ; 1 ;  1] | [5],
				[0 ; 0 ; 0 ;  6] | [6]
			]
		);
		assert_eq!(step3.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step3.row_sequence, vec![1, 2, 0, 3]);
		assert_eq!(step3.state, MatrixState::NormalizeRow(3));

		let step4 = step3.calculate_next().unwrap();
		assert_eq!(
			step4,
			matrix![
				[0 ; 1 ; 0 ; -1] | [1],
				[1 ; 1 ; 4 ;  2] | [3],
				[0 ; 2 ; 1 ;  1] | [5],
				[0 ; 0 ; 0 ;  1] | [1]
			]
		);
		assert_eq!(step4.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step4.row_sequence, vec![1, 2, 0, 3]);
		assert_eq!(step4.state, MatrixState::ReInsertRow(2));

		let step5 = step4.calculate_next().unwrap();
		assert_eq!(
			step5,
			matrix![
				[0 ; 1 ; 0 ; 0] | [2],
				[1 ; 1 ; 4 ; 2] | [3],
				[0 ; 2 ; 1 ; 1] | [5],
				[0 ; 0 ; 0 ; 1] | [1]
			]
		);
		assert_eq!(step5.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step5.row_sequence, vec![1, 2, 0, 3]);
		assert_eq!(step5.state, MatrixState::ReInsertRow(1));

		let step6 = step5.calculate_next().unwrap();
		assert_eq!(
			step6,
			matrix![
				[0 ; 1 ; 0 ; 0] | [2],
				[1 ; 1 ; 4 ; 2] | [3],
				[0 ; 0 ; 1 ; 0] | [0],
				[0 ; 0 ; 0 ; 1] | [1]
			]
		);
		assert_eq!(step6.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step6.row_sequence, vec![1, 2, 0, 3]);
		assert_eq!(step6.state, MatrixState::ReInsertRow(0));

		let step7 = step6.calculate_next().unwrap();
		assert_eq!(
			step7,
			matrix![
				[0 ; 1 ; 0 ; 0] | [ 2],
				[1 ; 0 ; 0 ; 0] | [-1],
				[0 ; 0 ; 1 ; 0] | [ 0],
				[0 ; 0 ; 0 ; 1] | [ 1]
			]
		);
		assert_eq!(step7.col_sequence, vec![0, 2, 1, 3]);
		assert_eq!(step7.row_sequence, vec![1, 2, 0, 3]);
		assert_eq!(step7.state, MatrixState::Done);

		assert_eq!(solve(initial.clone()), step7);
		assert_eq!(solve_with_history(initial).get(-1), step7);
	}
}
