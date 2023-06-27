#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec;
use core::cell::RefCell;
use core::future::Future;
use core::marker::PhantomData;
use core::task::Poll;

use futures::prelude::*;
use log::{debug, info, trace, warn};
use smoltcp::{
    iface::{self, Config, Context, Interface, SocketHandle, SocketSet},
    phy::{Device, Medium},
    socket::{dhcpv4, tcp, AnySocket},
    time::{Duration, Instant},
    wire::{HardwareAddress, IpCidr, Ipv4Address, Ipv4Cidr},
};

pub(crate) const DEFAULT_KEEP_ALIVE_INTERVAL: u64 = 75000;
pub(crate) const DEFAULT_TCP_SOCKET_BUFFER_SIZE: usize = 65535;

#[derive(Clone)]
pub struct SharedNetwork {
    inner: Rc<RefCell<SharedNetworkInner>>,
}

pub struct SharedNetworkInner {
    iface: Interface,
    socket_set: SocketSet<'static>,
    // TODO add static alternative
    dhcp_socket_handle: SocketHandle,
    // TODO add list of DNS servers
}

pub type TcpSocket = Socket<tcp::Socket<'static>>;

pub struct Socket<T> {
    handle: SocketHandle,
    shared: SharedNetwork,
    _phantom: PhantomData<T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TcpSocketError {
    InvalidState(tcp::State),
    RecvError(tcp::RecvError),
    SendError(tcp::SendError),
}

impl SharedNetwork {
    pub fn new<D: Device + ?Sized>(config: Config, device: &mut D) -> Self {
        let iface = Interface::new(config, device);
        let mut socket_set = SocketSet::new(vec![]);
        let dhcp_socket = dhcpv4::Socket::new();
        let dhcp_socket_handle = socket_set.add(dhcp_socket);
        Self {
            inner: Rc::new(RefCell::new(SharedNetworkInner {
                iface,
                socket_set,
                dhcp_socket_handle,
            })),
        }
    }

    pub fn inner(&self) -> &Rc<RefCell<SharedNetworkInner>> {
        &self.inner
    }

    pub fn new_tcp_socket(&self) -> TcpSocket {
        self.new_tcp_socket_with_buffer_sizes(
            DEFAULT_TCP_SOCKET_BUFFER_SIZE,
            DEFAULT_TCP_SOCKET_BUFFER_SIZE,
        )
    }

    pub fn new_tcp_socket_with_buffer_sizes(
        &self,
        rx_buffer_size: usize,
        tx_buffer_size: usize,
    ) -> TcpSocket {
        let rx_buffer = tcp::SocketBuffer::new(vec![0; rx_buffer_size]);
        let tx_buffer = tcp::SocketBuffer::new(vec![0; tx_buffer_size]);
        self.new_socket(tcp::Socket::new(rx_buffer, tx_buffer))
    }

    pub fn new_socket<T: AnySocket<'static>>(&self, socket: T) -> Socket<T> {
        let handle = self.inner.borrow_mut().socket_set.add(socket);
        Socket {
            handle,
            shared: self.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: AnySocket<'static>> Socket<T> {
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let network = self.shared.inner.borrow();
        let socket = network.socket_set.get(self.handle);
        f(socket)
    }

    pub fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut network = self.shared.inner.borrow_mut();
        let socket = network.socket_set.get_mut(self.handle);
        f(socket)
    }

    pub fn with_and_context_mut<R>(&mut self, f: impl FnOnce(&mut Context, &mut T) -> R) -> R {
        let network = &mut *self.shared.inner.borrow_mut();
        let context = network.iface.context();
        let socket = network.socket_set.get_mut(self.handle);
        f(context, socket)
    }
}

impl Socket<tcp::Socket<'static>> {
    pub async fn accept_with_keep_alive(
        &mut self,
        port: u16,
        keep_alive_interval: Option<Duration>,
    ) -> Result<(), TcpSocketError> {
        future::poll_fn(|cx| {
            self.with_mut(|socket| match socket.state() {
                tcp::State::Closed => {
                    socket.listen(port).unwrap();
                    Poll::Ready(())
                }
                tcp::State::Listen => Poll::Ready(()),
                _ => {
                    socket.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await;

        future::poll_fn(|cx| {
            self.with_mut(|socket| {
                if socket.is_active() {
                    Poll::Ready(Ok(()))
                } else {
                    let state = socket.state();
                    match state {
                        tcp::State::Closed
                        | tcp::State::Closing
                        | tcp::State::FinWait1
                        | tcp::State::FinWait2 => {
                            Poll::Ready(Err(TcpSocketError::InvalidState(state)))
                        }
                        _ => {
                            socket.register_recv_waker(cx.waker());
                            Poll::Pending
                        }
                    }
                }
            })
        })
        .await?;

        self.with_mut(|socket| socket.set_keep_alive(keep_alive_interval));

        Ok(())
    }

    pub async fn accept(&mut self, port: u16) -> Result<(), TcpSocketError> {
        self.accept_with_keep_alive(
            port,
            Some(Duration::from_millis(DEFAULT_KEEP_ALIVE_INTERVAL)),
        )
        .await
    }

    pub async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, TcpSocketError> {
        future::poll_fn(|cx| {
            self.with_mut(|socket| {
                if socket.can_recv() {
                    Poll::Ready(
                        socket
                            .recv_slice(buffer)
                            .map_err(TcpSocketError::RecvError)
                            .map(|n| {
                                assert!(n > 0);
                                n
                            }),
                    )
                } else {
                    let state = socket.state();
                    match state {
                        tcp::State::FinWait1
                        | tcp::State::FinWait2
                        | tcp::State::Closed
                        | tcp::State::Closing
                        | tcp::State::CloseWait
                        | tcp::State::TimeWait => {
                            Poll::Ready(Err(TcpSocketError::InvalidState(state)))
                        }
                        _ => {
                            socket.register_recv_waker(cx.waker());
                            Poll::Pending
                        }
                    }
                }
            })
        })
        .await
    }

    pub async fn send(&mut self, buffer: &[u8]) -> Result<(), TcpSocketError> {
        let mut pos = 0;

        while pos < buffer.len() {
            let n = future::poll_fn(|cx| {
                self.with_mut(|socket| {
                    if socket.can_send() {
                        Poll::Ready(
                            socket
                                .send_slice(&buffer[pos..])
                                .map_err(TcpSocketError::SendError),
                        )
                    } else {
                        let state = socket.state();

                        match state {
                            tcp::State::FinWait1
                            | tcp::State::FinWait2
                            | tcp::State::Closed
                            | tcp::State::Closing
                            | tcp::State::CloseWait
                            | tcp::State::TimeWait => {
                                Poll::Ready(Err(TcpSocketError::InvalidState(state)))
                            }
                            _ => {
                                socket.register_send_waker(cx.waker());
                                Poll::Pending
                            }
                        }
                    }
                })
            })
            .await?;

            assert!(n > 0);
            pos += n;
        }

        assert_eq!(pos, buffer.len());
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), TcpSocketError> {
        future::poll_fn(|cx| {
            self.with_mut(|socket| {
                let state = socket.state();
                match state {
                    tcp::State::FinWait1
                    | tcp::State::FinWait2
                    | tcp::State::Closed
                    | tcp::State::Closing
                    | tcp::State::TimeWait => Poll::Ready(Err(TcpSocketError::InvalidState(state))),
                    _ => {
                        if socket.send_queue() > 0 {
                            socket.register_send_waker(cx.waker());
                            Poll::Pending
                        } else {
                            socket.close();
                            Poll::Ready(Ok(()))
                        }
                    }
                }
            })
        })
        .await?;

        future::poll_fn(|cx| {
            self.with_mut(|socket| match socket.state() {
                tcp::State::FinWait1
                | tcp::State::FinWait2
                | tcp::State::Closed
                | tcp::State::Closing
                | tcp::State::TimeWait => Poll::Ready(()),
                _ => {
                    socket.register_send_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await;

        Ok(())
    }
}

impl<T> Drop for Socket<T> {
    fn drop(&mut self) {
        self.shared
            .inner
            .borrow_mut()
            .socket_set
            .remove(self.handle);
    }
}

impl SharedNetworkInner {
    pub fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.iface.poll_delay(timestamp, &mut self.socket_set)
    }

    pub fn poll<D: Device + ?Sized>(&mut self, timestamp: Instant, device: &mut D) -> bool {
        let activity = self.iface.poll(timestamp, device, &mut self.socket_set);
        if activity {
            self.poll_dhcp();
        }
        activity
    }

    // TODO should dhcp events instead just be monitored in a task?
    fn poll_dhcp(&mut self) {
        if let Some(event) = self
            .socket_set
            .get_mut::<dhcpv4::Socket>(self.dhcp_socket_handle)
            .poll()
        {
            match event {
                dhcpv4::Event::Configured(config) => {
                    info!("DHCP config acquired!");
                    info!("IP address:      {}", config.address);
                    self.iface.update_ip_addrs(|addrs| {
                        if let Some(dest) = addrs.iter_mut().next() {
                            *dest = IpCidr::Ipv4(config.address);
                        } else if addrs.push(IpCidr::Ipv4(config.address)).is_err() {
                            info!("Unable to update IP address");
                        }
                    });
                    if let Some(router) = config.router {
                        info!("Default gateway: {}", router);
                        self.iface
                            .routes_mut()
                            .add_default_ipv4_route(router)
                            .unwrap();
                    } else {
                        info!("Default gateway: None");
                        self.iface.routes_mut().remove_default_ipv4_route();
                    }

                    for (i, s) in config.dns_servers.iter().enumerate() {
                        info!("DNS server {}:    {}", i, s);
                    }
                }
                dhcpv4::Event::Deconfigured => {
                    info!("DHCP lost config!");
                    let cidr = Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0);
                    self.iface.update_ip_addrs(|addrs| {
                        if let Some(dest) = addrs.iter_mut().next() {
                            *dest = IpCidr::Ipv4(cidr);
                        }
                    });
                    self.iface.routes_mut().remove_default_ipv4_route();
                }
            }
        }
    }
}
