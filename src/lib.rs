#![feature(proc_macro_quote)]
mod fields;
mod quotes;
use anyhow::{bail, Result};
use fields::CornettoField;
use proc_macro2::TokenStream;
///!! This crate resolve a *mutable only in test* problem.
///!
///! # Example
///!
///! ```ignore
///! use cornetto::Cornetto;
///!
///! #[allow(dead_code)]
///! #[derive(Cornetto)]
///! struct Test {
///!     #[cornetto(mut, 200)] // mutable on test ( _reset(args...) )
///!     pub price: u64,
///!     #[cornetto(const, 150)] // always const
///!     pub const_price: u64,
///!     #[cornetto(mut, "youhouhou")]
///!     pub strin: String,
///! }
///!
///! fn main() {
///!     println!("{}", TEST.price() == 200);
///!     println!("{}", TEST.const_price() == 150);
///!     println!("{}", TEST.strin().eq("youhouhou"));
///!     // true, true and true
///! }
///!
///! #[cfg(test)]
///! mod test {
///!     #[test]
///!     fn test_cornetto() {
///!         super::TEST.price();
///!         assert_eq!(super::TEST.price(), 200);
///!         super::TEST._reset(100, "ho ho ho".to_string()); // only accessible from tests
///!         assert_eq!(super::TEST.price(), 100);
///!     }
///! }
///! ```
///!
///! # How to use it
///!
///! In the structure that you want to organise all your project constants, use
///! the derive Cornetto as in the example. Define if you want to be able to
///! mutate the identificator with keywords `const` and `mut`.
///!
///! You just created a lazy_static reference of an object that have the same
///! name of your structure but in uppercase.
///!
///! If you choose the `mut`, you'll be able to reset it with the `_reset`
///! function (only in test cfg).
///!
///! ```ignore
///! /// #[cfg(test)]
///! mod test {
///!     #[test]
///!     fn test_cornetto() {
///!         super::TEST.price();
///!         assert_eq!(super::TEST.price(), 200);
///!         super::TEST._reset(100, "ho ho ho".to_string()); // only accessible from tests
///!         assert_eq!(super::TEST.price(), 100);
///!     }
///! }
///! ```
///!
///! > Note that the parameters of the `_reset` are always the mutable
///! > identificators in order of declaration.
///!
///!
///! # How it works
///!
///! This procedural macro generate a new structure specially for to manage all
///! constants and mutables. For each parameters you also get a function
///! implemented for this structure named `Cornetto${Struct_name}`
///!
///! ```ignore
///! impl CornettoTest {
///!     #[cfg(test)]
///!     pub fn price(&self) -> u64 {
///!         self.p_fields.lock().unwrap().price
///!     }
///!     pub fn const_price(&self) -> u64 {
///!         DEFAULT_TEST_CONST_PRICE
///!     }
///!     #[cfg(test)]
///!     pub fn strin(&self) -> String {
///!         self.p_fields.lock().unwrap().strin.to_string()
///!     }
///!     #[cfg(test)]
///!     pub fn _reset(&self, price: u64, strin: String) {
///!         let mut lock = self.p_fields.lock().unwrap();
///!         lock.price = price;
///!         lock.strin = strin;
///!     }
///!     #[cfg(not(test))]
///!     pub fn price(&self) -> u64 {
///!         DEFAULT_TEST_PRICE
///!     }
///!     #[cfg(not(test))]
///!     pub fn strin(&self) -> String {
///!         DEFAULT_TEST_STRIN.to_string()
///!     }
///! }
///! ```
///!
///! ## Get default configuration
///!
///! You can allways get the default configuration on the first initialisation
///! of the cornetto object with the function bellow. The cornetto generated
///! structure implement the `Default` traits in order to initialize the static
///! reference.
///!
///! ```ignore
///! let def = CornettoTest::default()
///! ```
///!
///! ## Get constants generated
///!
///! You also generate with the derive cornetto some constants.
///! ```ignore
///! const DEFAULT_TEST_PRICE: u64 = 200;
///! const DEFAULT_TEST_CONST_PRICE: u64 = 150;
///! const DEFAULT_TEST_STRIN: &str = "youhouhou";
///! ```
///!
///!
///! ## Analyzer
///! > The hygien of the crate allow you to access to anything created by the
///! > derive. But if you want to allow your IDE to see the generated functions
///! > and strutures, we have to activate the `support for procedural macros`
///! > and the expand of `attribute macros`.

#[proc_macro_derive(Cornetto, attributes(cornetto))]
pub fn cornetto_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    impl_cornetto(&ast).unwrap().into()
}

fn impl_cornetto(input: &syn::DeriveInput) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        bail!("`#![derive(Cornetto)]` cannot be applied to types with generic parameters")
    }
    match &input.data {
        syn::Data::Struct(ds) => quotes::write(&input.ident, &CornettoField::parse(ds)?),
        _ => bail!("`#![derive(Cornetto)]` cannot be applied to other than structs"),
    }
}
