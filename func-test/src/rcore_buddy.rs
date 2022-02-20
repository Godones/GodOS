use std::alloc::{GlobalAlloc, Layout};
use std::time::Instant;
use buddy_system_allocator::LockedHeap;
use crate::config::DATA;

pub unsafe fn test()->u128{
    let begin = Instant::now();
    let data = DATA.as_mut_ptr();
    let buddy = LockedHeap::<32>::new();
    buddy.lock().init(data as usize, 16 * 5000);

    let layout = Layout::from_size_align(32, 32).unwrap();
    for _ in 0..400{
        let ptr = buddy.alloc(layout);
        buddy.dealloc(ptr, layout);
    }
    let mut prts = Vec::new();
    for _ in 0..400{
        let ptr = buddy.alloc(layout);
        prts.push(ptr);
    }
    for it in prts {
        buddy.dealloc(it, layout);
    }

    let layouts = vec![8,16,32,64,128,512];
    for _ in 0..1000{
        for j in &layouts{
            let layout = Layout::from_size_align(*j, *j).unwrap();
            let ptr = buddy.alloc(layout);
            buddy.dealloc(ptr, layout);
        }
    }
    let mut prts = Vec::new();
    for _ in 0..2000{
        let ptr = buddy.alloc(layout);
        prts.push(ptr);
    }
    for i in prts{
        buddy.dealloc(i, layout);
    }

    let end = begin.elapsed().as_micros();
    // println!("[rcore-buddy] test over, cost :{}",end);
    end
}