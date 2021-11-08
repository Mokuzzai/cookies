#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::content::Html;
use rocket::response::Redirect;
use rocket::*;

use anyhow::anyhow;
use anyhow::Result;

use serde_derive::Deserialize;
use serde_derive::Serialize;

use std::fs::File;

mod minesweeper {
	use super::*;

	pub const STATE: &'static str = "state";
	pub const DISPLAY: &'static str = "display";

	#[derive(Debug, FromForm)]
	pub struct Builder {
		width: usize,
		height: usize,
	}

	impl Builder {
		fn finish(&self) -> State {
			let width = self.width;
			let data = vec![Cell::default(); width * self.height].into_boxed_slice();

			State { width, data }
		}
	}

	#[derive(Debug, FromFormValue)]
	pub enum PlayKind {
		Open,
		Flag,
	}

	#[derive(Debug, FromForm)]
	pub struct Play {
		x: usize,
		y: usize,
		play: PlayKind,
	}

	#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
	enum CellClosed {
		Mine,
		Safe,
	}

	#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
	enum CellOpened {
		Mine,
		Safe(u8),
	}

	#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
	enum Cell {
		Closed(CellClosed),
		Opened(CellOpened),
	}

	impl Default for Cell {
		fn default() -> Self {
			Self::Closed(CellClosed::Safe)
		}
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct State {
		width: usize,
		data: Box<[Cell]>,
	}

	impl State {
		fn extents(&self) -> [usize; 2] {
			[self.width, self.data.len() / self.width]
		}
	}

	const _: () = {
		use core::fmt::*;

		impl Display for CellClosed {
			fn fmt(&self, f: &mut Formatter) -> Result {
				f.pad("[ ]")
			}
		}

		impl Display for CellOpened {
			fn fmt(&self, f: &mut Formatter) -> Result {
				match *self {
					Self::Mine => f.pad("{+}"),
					Self::Safe(0) => f.pad("   "),
					Self::Safe(n) if n > 8 => unreachable!(),
					Self::Safe(n) => write!(f, "[{}]", n),
				}
			}
		}

		impl Display for Cell {
			fn fmt(&self, f: &mut Formatter) -> Result {
				match *self {
					Self::Closed(ref closed) => Display::fmt(closed, f),
					Self::Opened(ref opened) => Display::fmt(opened, f),
				}
			}
		}

		impl Display for State {
			fn fmt(&self, f: &mut Formatter) -> Result {
				for row in self.data.chunks(self.width) {
					for cel in row {
						write!(f, "{}", cel)?;
					}
					write!(f, "\n")?;
				}

				Ok(())
			}
		}
	};

	impl State {
		fn commit(&mut self, play: &Play) {}
	}

	#[get("/minesweeper")]
	pub fn index(mut cookies: Cookies) -> Html<Result<File>> {
		Html(File::open("assets/minesweeper/index.html").map_err(Into::into))
	}

	#[post("/minesweeper/setup?", data="<builder>")]
	pub fn setup(builder: Form<Builder>, mut cookies: Cookies) -> Result<&'static str> {
		println!("{:?}", builder);

		let state = builder.finish();

		println!("{:?}", state.extents());

		cookies.add(Cookie::new(STATE, serde_json::to_string(&state)?));
		cookies.add(Cookie::new(DISPLAY, state.to_string()));

		// Ok(Redirect::to(uri!(render)))
		Ok("ok")
	}

	#[get("/minesweeper/playing")]
	pub fn render(mut cookies: Cookies) -> Option<String> {
		cookies.get(DISPLAY).map(|cookie| cookie.value().to_owned())
	}
}

pub enum Either<L, R> {
	Left(L),
	Right(R),
}

impl<'a, L, R> response::Responder<'a> for Either<L, R>
where
	L: response::Responder<'a>,
	R: response::Responder<'a>,
{
	fn respond_to(self, request: &Request<'_>) -> response::Result<'a> {
		match self {
			Self::Left(left) => left.respond_to(request),
			Self::Right(right) => right.respond_to(request),
		}
	}
}

fn main() {
	ignite()
		.mount(
			"/",
			routes![
				minesweeper::index,
				minesweeper::setup,
				minesweeper::render,
			],
		)
		.launch();
}
