use crate::config::RING_BUFFER_SIZE;
use crate::file::{File, Stat, StatMode};
use crate::mm::page_table::UserBuffer;
use alloc::sync::{Arc, Weak};
use spin::Mutex;
use crate::println;
use crate::task::suspend_current_run_next;

pub struct Pipe {
    readable: bool,
    writeable: bool,
    buffer: Arc<Mutex<RingBuffer>>,
}
#[derive(Copy, Clone,PartialEq)]
pub enum RingBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}
pub struct RingBuffer {
    status: RingBufferStatus,
    head: usize,
    tail: usize, //记录队列的头尾下标
    msg: [u8; RING_BUFFER_SIZE],
    // write_end 字段保存它的写端的一个弱引用计数，这
    // 由于确认所有的写端均关闭
    write_end: Option<Weak<Pipe>>,
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            status: RingBufferStatus::EMPTY,
            head: 0,
            tail: 0,
            msg: [0; RING_BUFFER_SIZE],
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        //设置写端的一个弱引用计数
        self.write_end = Some(Arc::downgrade(write_end));
    }
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::NORMAL;
        let data = self.msg[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::EMPTY;
        }
        data
    }
    pub fn available_read(&self) -> usize {
        //返回可读的内容数量
        if self.status == RingBufferStatus::EMPTY {
            0
        } else {
            if self.tail > self.head {
                self.tail - self.head
            } else {
                self.tail + RING_BUFFER_SIZE - self.head
            }
        }
    }
    pub fn write_byte(&mut self,val:u8){
        self.status = RingBufferStatus::NORMAL;
        self.msg[self.tail] = val;
        self.tail  = (self.tail+1)%RING_BUFFER_SIZE;
        if self.tail == self.head{
            self.status = RingBufferStatus::FULL;
        }
    }
    pub fn available_write(&self)->usize{
        if self.status== RingBufferStatus::FULL { 0 }
        else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }

    pub fn is_write_end_closed(&self)->bool{
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

impl Pipe {
    fn read_from_buffer(buffer: Arc<Mutex<RingBuffer>>) -> Self {
        //从环形缓冲区分配读端文件
        Self {
            readable: true,
            writeable: false,
            buffer,
        }
    }
    fn write_from_buffer(buffer: Arc<Mutex<RingBuffer>>) -> Self {
        Self {
            readable: false,
            writeable: true,
            buffer,
        }
    }
    pub fn new() -> (Arc<Pipe>, Arc<Pipe>) {
        let ringbuffer = Arc::new(Mutex::new(RingBuffer::new()));
        let read_end = Arc::new(Pipe::read_from_buffer(ringbuffer.clone()));
        let write_end = Arc::new(Pipe::write_from_buffer(ringbuffer.clone()));
        ringbuffer.lock().set_write_end(&write_end);
        (read_end, write_end)
    }
}

// todo!(检查读写是否正确)
impl File for Pipe {
    fn read(&self, buf: UserBuffer) -> usize {
        assert_eq!(self.readable, true);
        let mut read_size = 0 as usize;
        let mut bufiter = buf.into_iter();
        loop {
            let mut buffer = self.buffer.lock();
            let available_size = buffer.available_read();//查看可读数量
            if available_size==0 {
                if  buffer.is_write_end_closed(){
                    return read_size;//如果写端已经全部关闭，那么就不需要再等待
                }
                drop(buffer);
                suspend_current_run_next();//等待之后的写端往这里面写内容
                continue;
            }
            for _ in 0..available_size{
                if let Some(val) = bufiter.next(){
                    unsafe {
                        *val = buffer.read_byte();
                        read_size +=1;
                    }
                }
                else {
                    return read_size;
                }
            }
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        assert_eq!(self.writeable,true);
        let mut write_size = 0 as usize;
        let mut user_buf_iter = buf.into_iter();
        loop {
            let mut buffer = self.buffer.lock();
            let available_size = buffer.available_write();//查看可写数量
            if available_size==0 {
                drop(buffer);
                suspend_current_run_next();//等待之后的读端往这里面读内容
                continue;
            }
            for _ in 0..available_size{
                if let Some(val) = user_buf_iter.next(){
                    unsafe {
                        buffer.write_byte(*val);
                        write_size +=1;
                    }
                }
                else {
                    println!("write_size: {}",write_size);
                    return write_size;
                }
            }
        }
    }
    fn fstat(&self) -> Stat {
        Stat::new(
            0,
            0,
            StatMode::NULL,
            1,
        )
    }
}
