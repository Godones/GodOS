#![no_main]
#![no_std]
#![feature(asm)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lib;
use lib::yield_;
const WIDTH:usize = 10;
const HEIGHT:usize = 5;


#[no_mangle]
fn main()->i32{

    for i in 0..HEIGHT{
        println!("Print the word 'B': ");
        for _ in 0..WIDTH {
            print!("B");
        }
        println!("[{}/{}]",i+1,HEIGHT);
        yield_();//暂停应用
    }
    println!("Test print B is Ok");
    0
}