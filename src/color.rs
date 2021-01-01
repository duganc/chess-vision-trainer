

static WHITE_COLOR_CODE: &str = "\x1b[37m";
static RED_COLOR_CODE: &str = "\x1b[31m";
static GREEN_COLOR_CODE: &str = "\x1b[32m";
static RESET_COLOR_CODE: &str = "\x1b[0m";

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
	White,
	Red,
	Green,
}

impl Color {
	pub fn format(&self, s: String) -> String {
		let color_code = match self {
			Self::White => WHITE_COLOR_CODE,
			Self::Red => RED_COLOR_CODE,
			Self::Green => GREEN_COLOR_CODE,
		};
		format!("{}{}{}", color_code.to_string(), s, RESET_COLOR_CODE)
	}
}