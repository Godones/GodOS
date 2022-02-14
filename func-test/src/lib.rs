#![feature(alloc_error_handler)]
#![no_std]
#![feature(const_mut_refs)]
mod linked_list;
mod common;
extern crate alloc;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
