use matrix::Matrix;
use solver::MatrixSolver;

pub mod fraction;
pub mod matrix;
pub mod solver;
pub mod row;
pub mod parser;

pub use parser::parse;

pub fn solve(m: Matrix) -> Matrix {
	solve_with_history(m).get(-1)
}

pub fn solve_with_history(m: Matrix) -> MatrixSolver {
	let mut s = MatrixSolver::new(m);
	s.solve();
	s
}
