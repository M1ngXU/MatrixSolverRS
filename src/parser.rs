use crate::{fraction::Fraction, matrix::Matrix, row::Row};

/// format:
/// (a;b;c|e;f)
/// (g;h;i|j;k)
/// (l;m;n|o;p)
/// where each char can be a number or number/number (fraction)
pub fn parse(s: &str) -> Result<Matrix, String> {
	let mut rows = Vec::with_capacity(s.lines().count());
	for line in s.lines() {
		if !line.starts_with('(') {
			return Err(format!(
				"`{line}` doesn't start with a `(`."
			));
		}
		if !line.ends_with(')') {
			return Err(format!(
				"`{line}` doesn't end with a `)`."
			));
		}
		if let Some((left, right)) = line[1..line.len() - 1].split_once('|') {
			rows.push(Row::new(
				parse_side(left).and_then(|l| {
					if l.len() == rows.capacity() {
						Ok(l)
					} else {
						Err(format!(
							"`{line}` has not {} fractions.",
							rows.capacity()
						))
					}
				})?,
				parse_side(right)?,
			));
		} else {
			return Err(format!(
				"`{line}` doesn't have a `|`."
			));
		}
	}
	Ok(Matrix::create_with_rows(rows))
}

fn parse_side(s: &str) -> Result<Vec<Fraction>, String> {
	let mut left_nums = Vec::with_capacity(s.split(';').count());
	for f in s.split(';') {
		if let Some(fraction) = f.parse::<isize>().ok().map(Fraction::from).or_else(|| {
			f.split_once('/').and_then(|(l, r)| {
				l.parse::<isize>()
					.ok()
					.and_then(|l| r.parse::<isize>().ok().map(|r| (l, r)))
					.map(Fraction::from)
			})
		}) {
			left_nums.push(fraction);
		} else {
			return Err(format!(
				"Fraction `{f}` can't be parsed."
			));
		}
	}
	Ok(left_nums)
}

#[cfg(test)]
mod test {
	use crate::{matrix, parser::parse};

	#[test]
	fn simple() {
		assert_eq!(parse("(1|2)"), Ok(matrix![[1] | [2]]));
	}

	#[test]
	fn complex() {
		assert_eq!(
			parse("(1;3;2/3|2)\n(2;-4;0|9)\n(0/2;2/4;1/2|5)"),
			Ok(matrix![
				[1;3;(2,3)]|[2],
				[2;-4;0]|[9],
				[0;(1,2);(1,2)]|[5]
			])
		);
	}

    #[test]
    fn fails() {
        assert!(parse("1|3").is_err());
        assert!(parse("(2|3").is_err());
        assert!(parse("2|3)").is_err());
        assert!(parse("(2|3)\n(3;4|4)").is_err());
        assert!(parse("(2;3|").is_err());
        assert!(parse("(2;3)").is_err());
    }
}
