// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Utilities for server implementations

use crate::{analyzer_cache::AnalyzerCache, state::ServiceState, timeline_cache::TimelineCache};
use axum::Router as AxumRouter;
use quent_collector::server::CollectorService;
use quent_collector_proto::collector_server::CollectorServer;
use quent_query_engine_analyzer::ui::{QuentViewer, UiAnalyzer};

use tonic::transport::{Server as GrpcServer, server::Router};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

pub mod analyzer_cache;
pub mod error;
mod state;
mod timeline_cache;
mod ui;

/// Application-specific HTTP routes composed by a model viewer.
pub type ViewerRoutes = AxumRouter;

/// Server-side route composition owned by a model's viewer implementation.
///
/// This is separate from [`QuentViewer`] so existing viewers keep their
/// import/indexing contract. New viewers can opt into application-specific
/// routes while sharing the same analyzer cache as the common query-engine
/// endpoints.
pub trait QuentViewerServer: QuentViewer {
    /// Build application-specific routes from the common analyzer cache.
    fn additional_routes(analyzers: AnalyzerCache<Self::Analyzer>) -> ViewerRoutes;
}

pub fn initialize_tracing(log_level: &str) {
    use tracing_subscriber::{
        EnvFilter,
        fmt::{self, format::FmtSpan},
        layer::SubscriberExt,
        registry,
        util::SubscriberInitExt,
    };
    registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(format!("{log_level},h2=off,tonic=off"))),
        )
        .with(
            fmt::layer()
                .with_target(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stderr),
        )
        .init();
}

pub fn collector_service<C, F>(make: F) -> Result<Router, Box<dyn std::error::Error>>
where
    C: quent_collector::CollectorSink + Send + Sync + 'static,
    F: Fn(Uuid) -> Result<C, String> + Send + Sync + 'static,
{
    let collector = CollectorService::<C>::new(make);
    Ok(GrpcServer::builder().add_service(CollectorServer::new(collector)))
}

pub fn analyzer_service_router<A>(
    importer: Box<analyzer_cache::ImporterFn<A>>,
    lister: Box<analyzer_cache::ListerFn>,
    cors: Option<String>,
) -> Result<AxumRouter, Box<dyn std::error::Error>>
where
    A: UiAnalyzer + Send + Sync + 'static,
{
    analyzer_service_router_with_routes::<A>(importer, lister, cors, AxumRouter::new())
}

/// Build the complete HTTP router for a model-owned viewer.
pub fn model_viewer_router<V>(
    importer: Box<analyzer_cache::ImporterFn<V::Analyzer>>,
    lister: Box<analyzer_cache::ListerFn>,
    cors: Option<String>,
) -> Result<AxumRouter, Box<dyn std::error::Error>>
where
    V: QuentViewerServer,
    V::Analyzer: Send + Sync + 'static,
{
    analyzer_service_router_with_analyzer_routes::<V::Analyzer>(
        importer,
        lister,
        cors,
        V::additional_routes,
    )
}

/// Build the complete HTTP router while retaining metadata from each common
/// context load.
///
/// This is the context-aware counterpart to [`model_viewer_router`]. Existing
/// viewer wrappers can keep using the event-only importer; applications whose
/// auxiliary views distinguish missing and present-empty streams can opt into
/// this path.
pub fn model_viewer_router_with_contexts<V>(
    importer: Box<analyzer_cache::ContextImporterFn<V::Analyzer>>,
    lister: Box<analyzer_cache::ListerFn>,
    cors: Option<String>,
) -> Result<AxumRouter, Box<dyn std::error::Error>>
where
    V: QuentViewerServer,
    V::Analyzer: Send + Sync + 'static,
{
    let analyzers = AnalyzerCache::<V::Analyzer>::new_with_contexts(importer, lister);
    let integration_routes = V::additional_routes(analyzers.clone());
    analyzer_service_router_from_cache(analyzers, cors, integration_routes)
}

/// Build the analyzer router and merge integration-owned routes before common
/// CORS and embedded-UI fallback layers are installed.
pub fn analyzer_service_router_with_routes<A>(
    importer: Box<analyzer_cache::ImporterFn<A>>,
    lister: Box<analyzer_cache::ListerFn>,
    cors: Option<String>,
    additional_routes: AxumRouter,
) -> Result<AxumRouter, Box<dyn std::error::Error>>
where
    A: UiAnalyzer + Send + Sync + 'static,
{
    analyzer_service_router_with_analyzer_routes::<A>(importer, lister, cors, move |_| {
        additional_routes
    })
}

/// Build the analyzer router and let integration routes share its analyzer cache.
///
/// The callback runs once during router construction. Routes can retain the
/// cloned cache to render application-owned auxiliary models without importing
/// or analyzing the same contexts through a second pipeline.
pub fn analyzer_service_router_with_analyzer_routes<A>(
    importer: Box<analyzer_cache::ImporterFn<A>>,
    lister: Box<analyzer_cache::ListerFn>,
    cors: Option<String>,
    additional_routes: impl FnOnce(analyzer_cache::AnalyzerCache<A>) -> AxumRouter,
) -> Result<AxumRouter, Box<dyn std::error::Error>>
where
    A: UiAnalyzer + Send + Sync + 'static,
{
    let analyzers = AnalyzerCache::<A>::new(importer, lister);
    let integration_routes = additional_routes(analyzers.clone());
    analyzer_service_router_from_cache(analyzers, cors, integration_routes)
}

fn analyzer_service_router_from_cache<A>(
    analyzers: AnalyzerCache<A>,
    cors: Option<String>,
    integration_routes: AxumRouter,
) -> Result<AxumRouter, Box<dyn std::error::Error>>
where
    A: UiAnalyzer + Send + Sync + 'static,
{
    let state = ServiceState {
        analyzers,
        timelines: TimelineCache::new(),
    };

    let mut http_routes = axum::Router::new()
        .nest("/api/engines", ui::routes(state))
        .merge(integration_routes);

    #[cfg(feature = "swagger")]
    {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;
        let api = ui::ApiDoc::openapi();
        http_routes =
            http_routes.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));
    }

    if let Some(cors) = cors {
        let cors = CorsLayer::new()
            .allow_origin(cors.parse::<axum::http::HeaderValue>().unwrap())
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE]);
        http_routes = http_routes.layer(cors);
    }

    #[cfg(feature = "ui")]
    {
        http_routes = http_routes.fallback(axum::routing::get(ui::embedded::serve));
    }

    Ok(http_routes)
}
