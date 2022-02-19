#![no_main]
#![no_std]

use lib::ls;

#[no_mangle]
fn main() {
    ls();
}
