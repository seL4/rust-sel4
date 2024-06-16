//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;
use heapless::Deque;

#[derive(Debug, Clone)]
pub struct WriteBuffered<T, const WRITE_BUF_SIZE: usize = 256> {
    unbuffered: T,
    write_buffer: Deque<u8, WRITE_BUF_SIZE>,
}

impl<T, const WRITE_BUF_SIZE: usize> WriteBuffered<T, WRITE_BUF_SIZE> {
    pub fn new(unbuffered: T) -> Self {
        Self {
            unbuffered,
            write_buffer: Deque::new(),
        }
    }

    fn enqueue<E>(&mut self, v: u8) -> nb::Result<(), E> {
        match self.write_buffer.push_back(v) {
            Ok(()) => Ok(()),
            Err(_) => Err(nb::Error::WouldBlock),
        }
    }

    fn enqueue_if_would_block<E>(&mut self, err: nb::Error<E>, v: u8) -> nb::Result<(), E> {
        match err {
            err @ nb::Error::Other(_) => Err(err),
            nb::Error::WouldBlock => self.enqueue(v),
        }
    }
}

impl<T: serial::Write<u8>, const WRITE_BUF_SIZE: usize> WriteBuffered<T, WRITE_BUF_SIZE> {
    fn write_entire_buffer(&mut self) -> nb::Result<(), <Self as serial::ErrorType>::Error> {
        loop {
            if let Some(v) = self.write_buffer.front() {
                if let err @ Err(_) = self.unbuffered.write(*v) {
                    break err;
                }
            } else {
                break Ok(());
            }
            self.write_buffer.pop_front().unwrap();
        }
    }
}

impl<T: serial::ErrorType, const WRITE_BUF_SIZE: usize> serial::ErrorType
    for WriteBuffered<T, WRITE_BUF_SIZE>
{
    type Error = T::Error;
}

impl<T: serial::Read, const WRITE_BUF_SIZE: usize> serial::Read<u8>
    for WriteBuffered<T, WRITE_BUF_SIZE>
{
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.unbuffered.read()
    }
}

impl<T: serial::Write, const WRITE_BUF_SIZE: usize> serial::Write<u8>
    for WriteBuffered<T, WRITE_BUF_SIZE>
{
    fn write(&mut self, v: u8) -> nb::Result<(), Self::Error> {
        match self.write_entire_buffer() {
            Err(err) => self.enqueue_if_would_block(err, v),
            Ok(()) => match self.unbuffered.write(v) {
                Err(err) => self.enqueue_if_would_block(err, v),
                Ok(()) => Ok(()),
            },
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.write_entire_buffer()?;
        self.unbuffered.flush()
    }
}
