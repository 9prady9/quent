// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Python lifecycle binding for a model-specific collector server.

use std::net::{Ipv6Addr, SocketAddr, TcpListener};

use pyo3::{
    exceptions::{PyOSError, PyRuntimeError, PyValueError},
    prelude::*,
    types::PyAny,
};
use quent_collector::{
    CollectorSink,
    server::{CollectorService, FlushHandle},
};
use quent_collector_proto::collector_server::CollectorServer;
use tokio::{runtime::Runtime, sync::oneshot, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use uuid::Uuid;

/// Owns the runtime used to serve requests and complete shutdown.
struct ServerHandle {
    shutdown: oneshot::Sender<()>,
    runtime: Runtime,
    task: JoinHandle<Result<(), String>>,
    flush: FlushHandle,
}

impl ServerHandle {
    /// Stops the server and waits for exporter shutdown before dropping its runtime.
    fn close(self) -> Result<(), String> {
        let Self {
            shutdown,
            runtime,
            task,
            flush,
        } = self;
        let _ = shutdown.send(());
        let result = runtime.block_on(task).map_err(|error| error.to_string());
        flush.wait();
        result?
    }
}

/// Owns a collector server running on a dedicated Rust runtime.
///
/// Call `close()` or use `with` to wait for shutdown. Discarding the object
/// starts shutdown in the background without waiting for it to finish.
#[pyclass(name = "Collector")]
pub struct Collector {
    address: String,
    handle: Option<ServerHandle>,
}

impl Collector {
    /// Starts a server that builds one `C` per source context ID.
    ///
    /// The returned address uses `advertised_host`, or the bound IP if omitted.
    /// A wildcard bind requires an explicit advertised host.
    ///
    /// # Errors
    ///
    /// Returns a Python exception if an address is invalid or the server cannot start.
    pub fn start<C>(
        bind_address: &str,
        advertised_host: Option<&str>,
        make: impl Fn(Uuid) -> Result<C, String> + Send + Sync + 'static,
    ) -> PyResult<Self>
    where
        C: CollectorSink + Send + Sync + 'static,
    {
        let bind_address: SocketAddr = bind_address.parse().map_err(|error| {
            PyValueError::new_err(format!("invalid collector bind address: {error}"))
        })?;
        let host = match advertised_host {
            Some(host) if !host.is_empty() => host.to_owned(),
            Some(_) => return Err(PyValueError::new_err("advertised_host cannot be empty")),
            None if bind_address.ip().is_unspecified() => {
                return Err(PyValueError::new_err(
                    "advertised_host is required for a wildcard bind address",
                ));
            }
            None => bind_address.ip().to_string(),
        };
        let listener = TcpListener::bind(bind_address).map_err(PyOSError::new_err)?;
        let port = listener.local_addr().map_err(PyOSError::new_err)?.port();
        let uri_host = if host.parse::<Ipv6Addr>().is_ok() {
            format!("[{host}]")
        } else {
            host
        };
        let address = format!("http://{uri_host}:{port}");
        let uri: http::Uri = address
            .parse()
            .map_err(|_| PyValueError::new_err("invalid advertised_host"))?;
        if uri.host().is_none() || uri.port_u16() != Some(port) {
            return Err(PyValueError::new_err("invalid advertised_host"));
        }
        listener.set_nonblocking(true).map_err(PyOSError::new_err)?;

        // The listener and gRPC request deadlines require I/O and time drivers.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(PyRuntimeError::new_err)?;
        let listener = {
            let _entered = runtime.enter();
            tokio::net::TcpListener::from_std(listener).map_err(PyOSError::new_err)?
        };
        let collector = CollectorService::new(make);
        let flush = collector.flush_handle();
        let (shutdown, shutdown_rx) = oneshot::channel();
        let task = runtime.spawn(async move {
            Server::builder()
                .add_service(CollectorServer::new(collector))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .map_err(|error| error.to_string())
        });
        Ok(Self {
            address,
            handle: Some(ServerHandle {
                shutdown,
                runtime,
                task,
                flush,
            }),
        })
    }
}

#[pymethods]
impl Collector {
    #[getter]
    /// Returns the HTTP URI used by collector clients.
    fn address(&self) -> &str {
        &self.address
    }

    #[getter]
    /// Reports whether `close()` has been called.
    fn closed(&self) -> bool {
        self.handle.is_none()
    }

    /// Stops accepting streams and waits for exporter shutdown attempts to finish.
    ///
    /// Repeated calls have no effect. A server failure raises a Python exception.
    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        match self.handle.take() {
            Some(handle) => py
                .detach(|| handle.close())
                .map_err(PyRuntimeError::new_err),
            None => Ok(()),
        }
    }

    /// Returns this collector for use in a Python `with` statement.
    fn __enter__(slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf
    }

    /// Closes the collector when a Python `with` statement exits.
    fn __exit__(
        &mut self,
        py: Python<'_>,
        _exc_type: &Bound<'_, PyAny>,
        _exc_value: &Bound<'_, PyAny>,
        _traceback: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.close(py)
    }
}

/// Requests shutdown without waiting when the collector is discarded.
impl Drop for Collector {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            std::thread::spawn(move || {
                let _ = handle.close();
            });
        }
    }
}
