#[macro_use]
extern crate anyhow;

pub mod nvgimpl;
// pub mod nvgimpl_orig;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
