use matrix::Matrix;
use matrix_solver::MatrixSolver;

use crate::row::Row;

pub mod fraction;
pub mod matrix;
pub mod matrix_solver;
pub mod row;

pub fn solve(m: Matrix) -> Matrix {
	solve_with_history(m).get(-1)
}

pub fn solve_with_history(m: Matrix) -> MatrixSolver {
	let mut s = MatrixSolver::new(m);
	s.solve();
	s
}

#[macro_export]
macro_rules! matrix {
    ($([$($left:expr);+] | [$($right:expr);+]),+ $(,)?) => {{
        $crate::matrix::Matrix::new(vec![
            $(
                Row::new(vec![$($left.into(),)+], vec![$($right.into(),)+]),
            )+
        ])
    }};
}

fn main() {
	println!(
		"{}",
		solve_with_history(matrix![
			[2; 1; 7; -2] | [5],
			[0; 0; 2; 1] | [3],
			[2; 2; 1; 0] | [1],
			[0; 0; 1; 1] | [2],
        ])
	)
}
