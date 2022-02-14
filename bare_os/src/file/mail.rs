use super::File;
use crate::file::{Stat, StatMode};
use crate::{mm::page_table::UserBuffer, task::suspend_current_run_next};
use alloc::sync::Arc;
use spin::Mutex;

const MAX_REPORT_NUMBER: usize = 256;
const MAX_MAIL_NUMBER: usize = 16;
pub struct Mail {
    buffer: Arc<Mutex<RingBuffer>>,
}
#[derive(Copy, Clone, PartialEq)]
pub enum RingBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}
pub struct RingBuffer {
    status: RingBufferStatus,
    head: usize,
    tail: usize, //记录队列的头尾下标
    msg: [u8; MAX_REPORT_NUMBER * MAX_MAIL_NUMBER],
}

impl RingBuffer {
    pub fn new() -> Self {
        Self {
            status: RingBufferStatus::EMPTY,
            head: 0,
            tail: 0,
            msg: [0; MAX_MAIL_NUMBER * MAX_REPORT_NUMBER],
        }
    }
    pub fn read_byte(&mut self) -> &[u8] {
        self.status = RingBufferStatus::NORMAL;
        let began = self.head * MAX_REPORT_NUMBER;
        let end = began + MAX_REPORT_NUMBER;
        //读取一个报文长度的内容出来
        let data = &self.msg[began..end];

        self.head = (self.head + 1) % MAX_MAIL_NUMBER;
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
                self.tail + MAX_MAIL_NUMBER - self.head
            }
        }
    }
    pub fn write_byte(&mut self, val: &[u8]) {
        self.status = RingBufferStatus::NORMAL;
        let begin = self.tail * MAX_REPORT_NUMBER;
        let end = begin + MAX_REPORT_NUMBER;

        // self.msg[self.tail] = val;
        //拷贝报文数据
        for index in begin..end {
            if index > val.len() {
                break;
            }
            self.msg[index] = val[index];
        }
        self.tail = (self.tail + 1) % MAX_MAIL_NUMBER;
        if self.tail == self.head {
            self.status = RingBufferStatus::FULL;
        }
    }
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::FULL {
            MAX_MAIL_NUMBER
        } else {
            MAX_MAIL_NUMBER - self.available_read()
        }
    }
}

impl Mail {
    pub fn new() -> Arc<Mail> {
        Arc::new(Self {
            buffer: Arc::new(Mutex::new(RingBuffer::new())),
        })
    }
}

impl File for Mail {
    fn read(&self, buf: UserBuffer) -> usize {
        let mut read_size = 0 as usize;
        let mut user_buf_iter = buf.into_iter();
        loop {
            let mut buffer = self.buffer.lock();
            let available_size = buffer.available_read(); //查看可读数量
            if available_size == 0 {
                drop(buffer);
                suspend_current_run_next(); //等待之后的写端往这里面写内容
                continue;
            }
            let data = buffer.read_byte();
            for index in 0..MAX_REPORT_NUMBER {
                if let Some(val) = user_buf_iter.next() {
                    unsafe {
                        *val = data[index];
                        read_size += 1;
                    }
                } else {
                    return read_size;
                }
            }
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut write_size = 0 as usize;
        let mut user_buf_iter = buf.into_iter();
        loop {
            let mut buffer = self.buffer.lock();
            let available_size = buffer.available_write(); //查看可读数量
            if available_size == 0 {
                drop(buffer);
                suspend_current_run_next(); //等待之后的读端往这里面读内容
                continue;
            }
            let mut user_buffer_data = [0 as u8; MAX_REPORT_NUMBER];
            for index in 0..MAX_REPORT_NUMBER {
                if let Some(val) = user_buf_iter.next() {
                    unsafe {
                        user_buffer_data[index] = *val;
                        write_size += 1;
                    }
                } else {
                    buffer.write_byte(&user_buffer_data);
                    return write_size;
                }
            }
        }
    }
    fn fstat(&self) -> Stat {
        Stat::new(0, 0, StatMode::NULL, 1)
    }
}
