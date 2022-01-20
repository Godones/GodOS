use alloc::sync::Arc;
use crate::sync::{MutexBlock, MutexSpin, Semaphore};
use crate::task::processor::current_process;

/// 创建一个互斥资源锁
pub fn sys_mutex_create(blocking:bool)->isize{
    let process = current_process();
    let mut process_inner = process.get_inner_access();
    //从进程的加锁向量中找到一个空闲位置
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_,item)|{ item.is_none() })
        .map(|(id,_)|{ id }){
        //找到一个空闲位置
        process_inner.mutex_list[id] = if !blocking {
            //自旋锁
            Some(Arc::new(MutexSpin::new()))
        }
        else {
            //互斥锁
            Some(Arc::new(MutexBlock::new()))
        };
        id as isize
    }else {
        process_inner.mutex_list.push(
            if!blocking{
                Some(Arc::new(MutexSpin::new()))
            }else {
                Some(Arc::new(MutexBlock::new()))
            }
        );
        (process_inner.mutex_list.len()-1) as isize
    }
}

// 对进程拥有的某个资源进行加锁
pub fn sys_mutex_lock(lock_id:usize)->isize{
    let process = current_process();
    let process_inner = process.get_inner_access();
    let mutex = process_inner.mutex_list[lock_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
pub fn sys_mutex_unlock(lock_id:usize)->isize{
    let process = current_process();
    let process_inner = process.get_inner_access();
    let mutex = process_inner.mutex_list[lock_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(count:usize)->isize{
    let process = current_process();
    let mut process_inner = process.get_inner_access();
    //从进程的加锁向量中找到一个空闲位置
    if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_,item)|{ item.is_none() })
        .map(|(id,_)|{ id }){
        //找到一个空闲位置
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(count)));
        id as isize
    }else {
        process_inner.semaphore_list.push(Some(Arc::new(Semaphore::new(count))));
        (process_inner.semaphore_list.len()-1) as isize
    }
}

// 对进程拥有的某个资源进行加锁
pub fn sys_semaphore_p(sem_id:usize)->isize{
    let process = current_process();
    let process_inner = process.get_inner_access();
    let semaphore = process_inner.semaphore_list[sem_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    semaphore.P();
    0
}
pub fn sys_semaphore_v(sem_id:usize)->isize{
    let process = current_process();
    let process_inner = process.get_inner_access();
    let semaphore = process_inner.semaphore_list[sem_id].as_ref().unwrap().clone();
    drop(process_inner);
    drop(process);
    semaphore.V();
    0
}