
#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

use rocket::*;
use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::content::Html;

use anyhow::Result;
use anyhow::anyhow;

use serde_derive::Serialize;
use serde_derive::Deserialize;


mod minesweeper {
	use super::*;

	#[derive(FromForm)]
	pub struct Builder {
		width: usize,
		height: usize,
	}

	impl Builder {
		fn finish(&self) -> State {
			let width = self.width;
			let data = vec![0; width * self.height].into_boxed_slice();

			State {
				width,
				data,
			}
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

	pub struct Cell {

	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct State {
		width: usize,
		data: Box<[u8]>,
	}

	const _: () = {
		use core::fmt::*;

		impl Display for State {
			fn fmt(&self, f: &mut Formatter) -> Result {
				Ok(())
			}
		}
	};

	impl State {
		fn commit(&mut self, play: &Play) {
			todo!()
		}
	}

	#[get("/start?<builder..>")]
	pub fn start(builder: Form<Builder>, mut cookies: Cookies) -> Result<()> {
		let state = builder.finish();

		cookies.add(Cookie::new("state", serde_json::to_string(&state)?));

		Ok(())
	}

	#[get("/playing?<play..>")]
	pub fn playing(play: Form<Play>, mut cookies: Cookies) -> Result<String> {
		let cookie = cookies.get("state").ok_or(anyhow!("missing cookie: `state`"))?;

		let mut state: State = serde_json::from_str(cookie.value())?;

		state.commit(&*play);

		cookies.add(Cookie::new("state", serde_json::to_string(&state)?));

		Ok(state.to_string())
	}
}

fn main() {
	ignite()
		.mount("/minesweeper", routes![
			minesweeper::start,
			minesweeper::playing,
		])
		.launch();
}
