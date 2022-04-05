use std::{fmt::Display, ops::Index};

use crate::matrix::{Matrix, MatrixState};

pub struct MatrixSolver {
	matrices: Vec<Matrix>,
}
impl MatrixSolver {
	pub fn new(initial: Matrix) -> Self {
		Self {
			matrices: vec![initial],
		}
	}

	pub fn solve(&mut self) {
		while let Some(new_matrix) = self.matrices.last().and_then(Matrix::calculate_next) {
			self.matrices.push(new_matrix);
		}
	}

	pub fn get(mut self, index: isize) -> Matrix {
		if index < 0 {
			self.matrices
				.swap_remove(self.matrices.len() - index.abs() as usize)
		} else {
			self.matrices.swap_remove(index as usize)
		}
	}
}
impl Index<isize> for MatrixSolver {
	type Output = Matrix;

	fn index(&self, index: isize) -> &Self::Output {
		if index < 0 {
			&self.matrices[self.matrices.len() - index.abs() as usize]
		} else {
			&self.matrices[index as usize]
		}
	}
}
impl Display for MatrixSolver {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "Initial matrix:")?;
		for m in &self.matrices {
			writeln!(f, "{}\n\n{}:", m, m.state())?;
		}
		let last = &self[-1];
		if last.state() == &MatrixState::Done {
			let result = last
				.row_sequence()
				.iter()
				.map(|i| {
					let r = &last.rows()[*i];
					r.right()
						.iter()
						.enumerate()
						.map(|(i, f)| {
							(
								f.is_negative(),
								if f.sign() == 0 && r.right().len() > 1 {
									String::new()
								} else if i > 0 && f.as_f64().abs() == 1.0 {
									String::from(
										(b't' + (i % u8::MAX as usize) as u8 - 1) as char,
									)
								} else if i == 0 {
									f.to_string()
								} else {
									format!(
										"{}{}",
										f.abs(),
										(b't' + (i % u8::MAX as usize) as u8 - 1) as char
									)
								},
							)
						})
						.collect::<Vec<(bool, String)>>()
				})
				.collect::<Vec<Vec<(bool, String)>>>();
			let max = (0..result[0].len())
				.filter_map(|i| result.iter().map(|v| v[i].1.len()).max())
				.collect::<Vec<usize>>();
			for (i, r) in result.iter().enumerate() {
				writeln!(
					f,
					"x_{:0>width$} = {}",
					last.col_sequence()[i] + 1,
					r.iter()
						.enumerate()
						.map(|(i, s)| if i == 0 {
							format!("{:>width$}", s.1, width = max[i])
						} else if s.1.is_empty() {
							" ".repeat(max[i] + 3)
						} else {
							format!(
								" {} {:>width$}",
								if s.0 { "-" } else { "+" },
								s.1,
								width = max[i]
							)
						})
						.collect::<Vec<String>>()
						.join(""),
					width = result.len().to_string().len()
				)?;
			}
		} else {
			writeln!(f, "Failed to solve matrix.")?;
		}
		Ok(())
	}
}
