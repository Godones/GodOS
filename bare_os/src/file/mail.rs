use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use spin::Mutex;
use alloc::sync::{Arc};
use crate::mm::page_table::UserBuffer;
use super::File;

const MAX_REPORT_NUMBER :usize = 256;
const MAX_MAIL_NUMBER:usize = 16;
pub struct Mail{
    buffer:Arc<Mutex<RingBuffer>>
}

pub struct RingBuffer{
    massage:VecDeque<Box<[u8]>>,
}

impl RingBuffer {
    pub fn new()->Self{
        Self{
            massage:VecDeque::new(),
        }
    }
    fn read_report(&mut self)->Box<[u8]>{
        //从缓冲区弹出一条报文
        self.massage.pop_front().unwrap()
    }
    fn is_availabel_read(&self)->bool{
        !self.massage.is_empty() //查看是否有报文存在
    }
    fn is_availabel_write(&self)->bool{
        self.massage.len()<MAX_MAIL_NUMBER //判断是否已经达到最大报文数量
    }
    fn write_report(&mut self,data:Box<[u8]>)->usize{
       self.massage.push_back(data);
       data.len()
    }

}

impl Mail {
    pub fn new()->Arc<Mail>{
        Arc::new(Self{
            buffer:Arc::new(Mutex::new(RingBuffer::new()))
        })
    }
}
impl File for Mail {
    fn write(&self, buf:UserBuffer) -> usize {
        let mut mail_buffer = self.buffer.lock();
        let mut write_size = 0;
        let user_buffer_len = buf.len();

        if mail_buffer.is_availabel_write(){
            let mut user_buffer_iter = buf.into_iter();
            let mut data = [0 as u8;MAX_REPORT_NUMBER];//先读取缓冲区的内容
            for index in 0..MAX_REPORT_NUMBER{
                if let Some(val) = user_buffer_iter.next(){
                    unsafe{data[index] = *val;}
                }
            }
            
            if user_buffer_len>MAX_REPORT_NUMBER{
                write_size = mail_buffer.write_report(Box::new(data));
            }
            else {
                
                write_size = mail_buffer.write_report(Box::new(data));
            }
        }
        write_size
    }

    fn read(&self, buf:UserBuffer) -> usize {
        let mail_buffer = self.buffer.lock();
        let mut read_size=  0;
        if mail_buffer.is_availabel_read(){
            let report = mail_buffer.read_report();//获取报文内容

            let mut user_buffer_iter = buf.into_iter();
            for index in 0..report.len(){
                if let Some(val) = user_buffer_iter.next(){
                    unsafe {
                        *val = report[index];
                        read_size +=1;
                    }
                }
                else {
                    return read_size;//没有报文读取
                }
            }
        }
        read_size
    } 
}
