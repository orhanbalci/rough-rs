#[macro_use]
extern crate derive_builder;

pub mod core;
pub mod filler;
pub mod generator;
pub mod geometry;
pub mod renderer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
