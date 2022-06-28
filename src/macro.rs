// eswm -- Emacs Standalown WindowManager
// Copyright (C) 2022 Jacob Stannix

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn rule(item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let item: Vec<&str> = item.as_str().split(',').collect();
    let (good_string, _) = item[3].split_at(item[3].len() - 1);
    let formatted = format!(
        "ParseRule {{ prefix: {}, infix: {}, precedence: {}}}",
        item[1], item[2], good_string
    );
    formatted.parse().unwrap()
}
