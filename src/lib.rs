#[macro_use]
extern crate anyhow;

pub mod nvgimpl;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
