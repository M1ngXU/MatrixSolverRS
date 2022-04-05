use matrix_solver::{solve_with_history, matrix};

fn main() {
	println!(
		"{}",
		solve_with_history(matrix![
			[1;1;1]|[1],
			[1;1;32]|[342],
			[2;8;5]|[124],
		])
	)
}