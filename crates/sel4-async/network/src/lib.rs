//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{self, Poll};

use futures::prelude::*;
use log::info;
use smoltcp::{
    iface::{Config, Context, Interface, SocketHandle, SocketSet},
    phy::Device,
    socket::{dhcpv4, dns, tcp, AnySocket},
    time::{Duration, Instant},
    wire::{DnsQueryType, IpAddress, IpCidr, IpEndpoint, IpListenEndpoint, Ipv4Address, Ipv4Cidr},
};

use sel4_async_network_traits::AsyncIO;

pub(crate) const DEFAULT_KEEP_ALIVE_INTERVAL: u64 = 75000;
pub(crate) const DEFAULT_TCP_SOCKET_BUFFER_SIZE: usize = 65535;

#[derive(Clone)]
pub struct ManagedInterface {
    inner: Rc<RefCell<ManagedInterfaceShared>>,
}

struct ManagedInterfaceShared {
    iface: Interface,
    socket_set: SocketSet<'static>,
    dns_socket_handle: SocketHandle,
    dhcp_socket_handle: SocketHandle,
    dhcp_overrides: DhcpOverrides,
}

#[derive(Default)]
pub struct DhcpOverrides {
    pub address: Option<Ipv4Cidr>,
    pub router: Option<Option<Ipv4Address>>,
    pub dns_servers: Option<Vec<Ipv4Address>>,
}

pub type TcpSocket = Socket<tcp::Socket<'static>>;

pub struct Socket<T> {
    handle: SocketHandle,
    shared: ManagedInterface,
    _phantom: PhantomData<T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TcpSocketError {
    InvalidState(tcp::State), // TODO just use InvalidState variants of below errors?
    RecvError(tcp::RecvError),
    SendError(tcp::SendError),
    ConnectError(tcp::ConnectError),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DnsError {
    StartQueryError(dns::StartQueryError),
    GetQueryResultError(dns::GetQueryResultError),
}

impl ManagedInterface {
    pub fn new<D: Device + ?Sized>(
        config: Config,
        dhcp_overrides: DhcpOverrides,
        device: &mut D,
        instant: Instant,
    ) -> Self {
        let iface = Interface::new(config, device, instant);
        let mut socket_set = SocketSet::new(vec![]);
        let dns_socket = dns::Socket::new(&[], vec![]);
        let dns_socket_handle = socket_set.add(dns_socket);
        let dhcp_socket = dhcpv4::Socket::new();
        let dhcp_socket_handle = socket_set.add(dhcp_socket);

        let mut this = ManagedInterfaceShared {
            iface,
            socket_set,
            dns_socket_handle,
            dhcp_socket_handle,
            dhcp_overrides,
        };

        this.apply_dhcp_overrides();

        Self {
            inner: Rc::new(RefCell::new(this)),
        }
    }

    fn inner(&self) -> &Rc<RefCell<ManagedInterfaceShared>> {
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
        let handle = self.inner().borrow_mut().socket_set.add(socket);
        Socket {
            handle,
            shared: self.clone(),
            _phantom: PhantomData,
        }
    }

    pub fn poll_at(&self, timestamp: Instant) -> Option<Instant> {
        self.inner().borrow_mut().poll_at(timestamp)
    }

    pub fn poll_delay(&self, timestamp: Instant) -> Option<Duration> {
        self.inner().borrow_mut().poll_delay(timestamp)
    }

    pub fn poll<D: Device + ?Sized>(&self, timestamp: Instant, device: &mut D) -> bool {
        self.inner().borrow_mut().poll(timestamp, device)
    }

    pub async fn dns_query(
        &self,
        name: &str,
        query_type: DnsQueryType,
    ) -> Result<Vec<IpAddress>, DnsError> {
        let query_handle = {
            let inner = &mut *self.inner().borrow_mut();
            inner
                .socket_set
                .get_mut::<dns::Socket>(inner.dns_socket_handle)
                .start_query(inner.iface.context(), name, query_type)
                .map_err(DnsError::StartQueryError)?
        };
        future::poll_fn(|cx| {
            let inner = &mut *self.inner().borrow_mut();
            let socket = inner
                .socket_set
                .get_mut::<dns::Socket>(inner.dns_socket_handle);
            match socket.get_query_result(query_handle) {
                Err(dns::GetQueryResultError::Pending) => {
                    socket.register_query_waker(query_handle, cx.waker());
                    Poll::Pending
                }
                r => Poll::Ready(
                    r.map(|heapless_vec| heapless_vec.to_vec())
                        .map_err(DnsError::GetQueryResultError),
                ),
            }
        })
        .await
    }
}

impl<T: AnySocket<'static>> Socket<T> {
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let network = self.shared.inner().borrow();
        let socket = network.socket_set.get(self.handle);
        f(socket)
    }

    pub fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut network = self.shared.inner().borrow_mut();
        let socket = network.socket_set.get_mut(self.handle);
        f(socket)
    }

    pub fn with_and_context_mut<R>(&mut self, f: impl FnOnce(&mut Context, &mut T) -> R) -> R {
        let network = &mut *self.shared.inner().borrow_mut();
        let context = network.iface.context();
        let socket = network.socket_set.get_mut(self.handle);
        f(context, socket)
    }
}

impl Socket<tcp::Socket<'static>> {
    pub async fn connect<T, U>(
        &mut self,
        remote_endpoint: T,
        local_endpoint: U,
    ) -> Result<(), TcpSocketError>
    where
        T: Into<IpEndpoint>,
        U: Into<IpListenEndpoint>,
    {
        self.with_and_context_mut(|cx, socket| socket.connect(cx, remote_endpoint, local_endpoint))
            .map_err(TcpSocketError::ConnectError)?;

        future::poll_fn(|cx| {
            self.with_mut(|socket| {
                let state = socket.state();
                match state {
                    tcp::State::Closed | tcp::State::TimeWait => {
                        Poll::Ready(Err(TcpSocketError::InvalidState(state)))
                    }
                    tcp::State::Listen => {
                        // TODO handle differently
                        Poll::Ready(Err(TcpSocketError::InvalidState(state)))
                    }
                    tcp::State::SynSent | tcp::State::SynReceived => {
                        socket.register_send_waker(cx.waker());
                        Poll::Pending
                    }
                    _ => Poll::Ready(Ok(())),
                }
            })
        })
        .await
    }

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

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, TcpSocketError> {
        future::poll_fn(|cx| self.poll_recv(cx, buffer)).await
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn poll_recv(
        &mut self,
        cx: &mut task::Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<Result<usize, TcpSocketError>> {
        self.with_mut(|socket| {
            if socket.can_recv() {
                Poll::Ready(
                    socket
                        .recv_slice(buffer)
                        .map_err(TcpSocketError::RecvError)
                        .map(|n| {
                            assert!(n > 0); // check assumption about smoltcp
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
                    | tcp::State::TimeWait => Poll::Ready(Err(TcpSocketError::InvalidState(state))),
                    _ => {
                        socket.register_recv_waker(cx.waker());
                        Poll::Pending
                    }
                }
            }
        })
    }

    pub async fn send_all(&mut self, buffer: &[u8]) -> Result<(), TcpSocketError> {
        let mut pos = 0;
        while pos < buffer.len() {
            let n = self.send(&buffer[pos..]).await?;
            assert!(n > 0);
            pos += n;
        }
        assert_eq!(pos, buffer.len());
        Ok(())
    }

    pub async fn send(&mut self, buffer: &[u8]) -> Result<usize, TcpSocketError> {
        future::poll_fn(|cx| self.poll_send(cx, buffer)).await
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn poll_send(
        &mut self,
        cx: &mut task::Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, TcpSocketError>> {
        self.with_mut(|socket| {
            if socket.can_send() {
                Poll::Ready(socket.send_slice(buffer).map_err(TcpSocketError::SendError))
            } else {
                let state = socket.state();
                match state {
                    tcp::State::FinWait1
                    | tcp::State::FinWait2
                    | tcp::State::Closed
                    | tcp::State::Closing
                    | tcp::State::CloseWait
                    | tcp::State::TimeWait => Poll::Ready(Err(TcpSocketError::InvalidState(state))),
                    _ => {
                        socket.register_send_waker(cx.waker());
                        Poll::Pending
                    }
                }
            }
        })
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

    pub fn abort(&mut self) {
        self.with_mut(|socket| socket.abort())
    }
}

impl AsyncIO for Socket<tcp::Socket<'static>> {
    type Error = TcpSocketError;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.poll_recv(cx, buf)
    }

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.poll_send(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // TODO
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // TODO
        Poll::Ready(Ok(()))
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

impl ManagedInterfaceShared {
    fn dhcp_socket_mut(&mut self) -> &mut dhcpv4::Socket<'static> {
        self.socket_set.get_mut(self.dhcp_socket_handle)
    }

    fn dns_socket_mut(&mut self) -> &mut dns::Socket<'static> {
        self.socket_set.get_mut(self.dns_socket_handle)
    }

    fn poll_at(&mut self, timestamp: Instant) -> Option<Instant> {
        self.iface.poll_at(timestamp, &self.socket_set)
    }

    fn poll_delay(&mut self, timestamp: Instant) -> Option<Duration> {
        self.iface.poll_delay(timestamp, &self.socket_set)
    }

    fn poll<D: Device + ?Sized>(&mut self, timestamp: Instant, device: &mut D) -> bool {
        let activity = self.iface.poll(timestamp, device, &mut self.socket_set);
        if activity {
            self.poll_dhcp();
        }
        activity
    }

    // TODO should dhcp events instead just be monitored in a task?
    fn poll_dhcp(&mut self) {
        if let Some(event) = self.dhcp_socket_mut().poll() {
            let event = free_dhcp_event(event);
            match event {
                dhcpv4::Event::Configured(config) => {
                    info!("DHCP config acquired");
                    if self.dhcp_overrides.address.is_none() {
                        self.set_address(config.address);
                    }
                    if self.dhcp_overrides.router.is_none() {
                        self.set_router(config.router);
                    }
                    if self.dhcp_overrides.dns_servers.is_none() {
                        self.set_dns_servers(&config.dns_servers);
                    }
                }
                dhcpv4::Event::Deconfigured => {
                    info!("DHCP config lost");
                    if self.dhcp_overrides.address.is_none() {
                        self.clear_address();
                    }
                    if self.dhcp_overrides.router.is_none() {
                        self.clear_router();
                    }
                    if self.dhcp_overrides.dns_servers.is_none() {
                        self.clear_dns_servers();
                    }
                }
            }
        }
    }

    fn set_address(&mut self, address: Ipv4Cidr) {
        let address = IpCidr::Ipv4(address);
        info!("IP address: {}", address);
        self.iface.update_ip_addrs(|addrs| {
            if let Some(dest) = addrs.iter_mut().next() {
                *dest = address;
            } else {
                addrs.push(address).unwrap();
            }
        });
    }

    fn clear_address(&mut self) {
        let cidr = Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0);
        self.iface.update_ip_addrs(|addrs| {
            if let Some(dest) = addrs.iter_mut().next() {
                *dest = IpCidr::Ipv4(cidr);
            }
        });
    }

    fn set_router(&mut self, router: Option<Ipv4Address>) {
        if let Some(router) = router {
            info!("Default gateway: {}", router);
            self.iface
                .routes_mut()
                .add_default_ipv4_route(router)
                .unwrap();
        } else {
            info!("Default gateway: (none)");
            self.iface.routes_mut().remove_default_ipv4_route();
        }
    }

    fn clear_router(&mut self) {
        self.iface.routes_mut().remove_default_ipv4_route();
    }

    fn set_dns_servers(&mut self, dns_servers: &[Ipv4Address]) {
        for (i, s) in dns_servers.iter().enumerate() {
            info!("DNS server {}: {}", i, s);
        }
        let dns_servers = dns_servers
            .iter()
            .copied()
            .map(From::from)
            .collect::<Vec<_>>();
        self.dns_socket_mut().update_servers(&dns_servers);
    }

    fn clear_dns_servers(&mut self) {
        self.dns_socket_mut().update_servers(&[]);
    }

    fn apply_dhcp_overrides(&mut self) {
        if let Some(address) = self.dhcp_overrides.address {
            self.set_address(address);
        }
        if let Some(router) = self.dhcp_overrides.router {
            self.set_router(router);
        }
        if let Some(dns_servers) = self.dhcp_overrides.dns_servers.clone() {
            // lazy, appease borrow checker
            self.set_dns_servers(&dns_servers);
        }
    }
}

fn free_dhcp_event(event: dhcpv4::Event) -> dhcpv4::Event<'static> {
    match event {
        dhcpv4::Event::Deconfigured => dhcpv4::Event::Deconfigured,
        dhcpv4::Event::Configured(config) => dhcpv4::Event::Configured(free_dhcp_config(config)),
    }
}

fn free_dhcp_config(config: dhcpv4::Config) -> dhcpv4::Config<'static> {
    dhcpv4::Config {
        server: config.server,
        address: config.address,
        router: config.router,
        dns_servers: config.dns_servers,
        packet: None,
    }
}
