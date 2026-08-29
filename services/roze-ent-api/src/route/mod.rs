#![allow(unused_imports)]

mod pets;
mod users;

use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    },
};

use roze_error::RozeError;
use roze_http::{
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, post},
    Json, Router,
};
use roze_result::ApiResponse;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::openapi;
use crate::svc::ServiceContext;

pub const WEBSOCKET_PUBLIC_ROUTES: &[&str] = &[];

pub fn router(ctx: ServiceContext) -> Router {
    let timeout = ctx
        .config
        .rest
        .as_ref()
        .filter(|rest| rest.middlewares.timeout)
        .and(ctx.config.governance.timeout_ms);
    let router = Router::new()
        .route("/api/v1/healthz", get(health))
        .route("/api/v1/readyz", get(readiness))
        .route("/api/v1/startupz", get(startup))
        .route("/api/v1/metrics", get(metrics))
        .route("/api/v1/reports/exports", post(create_report_export))
        .route(
            "/api/v1/reports/exports/{id}",
            get(report_export_status).delete(cancel_report_export),
        )
        .route("/api/v1/charts/query", post(chart_query))
        .route("/api/v1/openapi.json", get(openapi_doc))
        .merge(pets::routes())
        .merge(users::routes());
    let router = match timeout {
        Some(timeout_ms) => roze_middleware::apply_timeout(router, timeout_ms),
        None => router,
    };
    router.with_state(ctx)
}

async fn health(
    State(ctx): State<ServiceContext>,
) -> Result<ApiResponse<roze_health::ProbeReport>, RozeError> {
    Ok(ApiResponse::ok(
        ctx.health
            .liveness_report()
            .await
            .probe(roze_health::ProbeKind::Liveness),
    ))
}

async fn readiness(
    State(ctx): State<ServiceContext>,
) -> Result<ApiResponse<roze_health::ProbeReport>, RozeError> {
    Ok(ApiResponse::ok(
        ctx.health
            .readiness_report()
            .await
            .probe(roze_health::ProbeKind::Readiness),
    ))
}

async fn startup(
    State(ctx): State<ServiceContext>,
) -> Result<ApiResponse<roze_health::ProbeReport>, RozeError> {
    Ok(ApiResponse::ok(
        ctx.health
            .startup_report()
            .await
            .probe(roze_health::ProbeKind::Startup),
    ))
}

async fn metrics() -> String {
    roze_metrics::http_metrics()
}

async fn openapi_doc() -> Json<serde_json::Value> {
    Json(openapi::document())
}

const MAX_REPORT_COLUMNS: usize = 128;
const MAX_CHART_DIMENSIONS: usize = 8;
const MAX_CHART_MEASURES: usize = 16;
const MAX_QUERY_LIMIT: u64 = 10_000;
const MAX_ACTIVE_REPORT_EXPORTS: usize = 10_000;
const REPORT_EXPORT_EXPIRY_SECS: u64 = 15 * 60;
const REPORT_EXPORT_RETENTION_MILLIS: u64 = REPORT_EXPORT_EXPIRY_SECS * 1_000;
const CHART_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
struct ReportExportRequest {
    report: String,
    #[serde(default = "default_report_format")]
    format: String,
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    filters: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReportExportResource {
    id: String,
    report: String,
    format: String,
    status: String,
    progress_percent: u8,
    object_key: Option<String>,
    download_url: Option<String>,
    expires_at: Option<String>,
    error: Option<String>,
    from: Option<String>,
    to: Option<String>,
    timezone: Option<String>,
    column_count: usize,
    filter_count: usize,
    tenant_id: String,
    #[serde(skip_serializing)]
    owner_subject: String,
    #[serde(skip_serializing)]
    cancellation: roze_report::ReportCancellation,
    #[serde(skip_serializing)]
    terminal_at_millis: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChartQueryRequest {
    chart: String,
    #[serde(default)]
    dimensions: Vec<String>,
    #[serde(default)]
    measures: Vec<String>,
    #[serde(default)]
    filters: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    group_by: Vec<String>,
    #[serde(default)]
    sort: Vec<ChartSort>,
    #[serde(default)]
    time_bucket: Option<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default = "default_query_limit")]
    limit: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ChartSort {
    field: String,
    #[serde(default = "default_sort_direction")]
    direction: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChartQueryResponse {
    chart: String,
    dimensions: Vec<String>,
    measures: Vec<String>,
    time_bucket: Option<String>,
    timezone: Option<String>,
    from: Option<String>,
    to: Option<String>,
    filter_count: usize,
    scanned_rows: u64,
    result_rows: u64,
    series: Vec<ChartSeries>,
}

#[derive(Debug, Clone, Serialize)]
struct ChartSeries {
    name: String,
    points: Vec<ChartPoint>,
}

#[derive(Debug, Clone, Serialize)]
struct ChartPoint {
    timestamp: String,
    value: f64,
    labels: BTreeMap<String, String>,
}

fn default_report_format() -> String {
    "csv".to_string()
}

fn default_query_limit() -> u64 {
    1_000
}

fn default_sort_direction() -> String {
    "asc".to_string()
}

fn report_exports() -> &'static RwLock<BTreeMap<String, ReportExportResource>> {
    static EXPORTS: OnceLock<RwLock<BTreeMap<String, ReportExportResource>>> = OnceLock::new();
    EXPORTS.get_or_init(|| RwLock::new(BTreeMap::new()))
}

fn report_now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn validate_report_request(request: &ReportExportRequest) -> Result<(), RozeError> {
    if request.report.trim().is_empty() {
        return Err(RozeError::BadRequest("report is required".to_string()));
    }
    if !matches!(request.format.as_str(), "csv" | "xlsx") {
        return Err(RozeError::BadRequest(
            "format must be csv or xlsx".to_string(),
        ));
    }
    if request.columns.len() > MAX_REPORT_COLUMNS {
        return Err(RozeError::BadRequest("too many report columns".to_string()));
    }
    if request.filters.len() > MAX_REPORT_COLUMNS {
        return Err(RozeError::BadRequest("too many report filters".to_string()));
    }
    Ok(())
}

fn report_identity(
    headers: &HeaderMap,
    ctx: &ServiceContext,
) -> Result<(String, String), RozeError> {
    let Some(jwt) = ctx.jwt_config() else {
        return Ok(("anonymous".to_string(), "public".to_string()));
    };
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(RozeError::Unauthorized)?;
    let token = roze_jwt::extract_bearer_token(header).ok_or(RozeError::Unauthorized)?;
    let claims = roze_jwt::verify_token(token, &jwt).map_err(|_| RozeError::Unauthorized)?;
    let tenant = claims.tenant.ok_or(RozeError::Forbidden)?;
    Ok((claims.sub, tenant))
}

fn report_object_part(value: &str) -> String {
    let value = value
        .chars()
        .take(96)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if value.is_empty() {
        "unknown".to_string()
    } else {
        value
    }
}

fn ensure_export_owner(
    export: &ReportExportResource,
    subject: &str,
    tenant: &str,
) -> Result<(), RozeError> {
    if export.owner_subject == subject && export.tenant_id == tenant {
        Ok(())
    } else {
        Err(RozeError::Forbidden)
    }
}

async fn create_report_export(
    State(ctx): State<ServiceContext>,
    headers: HeaderMap,
    Json(request): Json<ReportExportRequest>,
) -> Result<ApiResponse<ReportExportResource>, RozeError> {
    validate_report_request(&request)?;
    let (subject, tenant_id) = report_identity(&headers, &ctx)?;
    let report_source = ctx
        .report_source()
        .map_err(|error| RozeError::Unavailable(error.to_string()))?;
    static EXPORT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let id = format!("export-{}", EXPORT_SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let cancellation = roze_report::ReportCancellation::new();
    let task_query = roze_report::ReportDataQuery {
        report: request.report.clone(),
        columns: request.columns.clone(),
        filters: request.filters.clone(),
        from: request.from.clone(),
        to: request.to.clone(),
        timezone: request.timezone.clone(),
        max_rows: roze_report::ExportLimits::default().max_rows,
    };
    let resource = ReportExportResource {
        id: id.clone(),
        report: request.report,
        format: request.format,
        status: "accepted".to_string(),
        progress_percent: 0,
        object_key: None,
        download_url: None,
        expires_at: None,
        error: None,
        from: request.from,
        to: request.to,
        timezone: request.timezone,
        column_count: request.columns.len(),
        filter_count: request.filters.len(),
        tenant_id: tenant_id.clone(),
        owner_subject: subject.clone(),
        cancellation: cancellation.clone(),
        terminal_at_millis: None,
    };
    {
        let mut exports = report_exports().write().await;
        let now = report_now_millis();
        exports.retain(|_, export| {
            export.terminal_at_millis.is_none_or(|terminal| {
                now.saturating_sub(terminal) < REPORT_EXPORT_RETENTION_MILLIS
            })
        });
        if exports.len() >= MAX_ACTIVE_REPORT_EXPORTS {
            return Err(RozeError::Unavailable(
                "report export capacity is exhausted".to_string(),
            ));
        }
        exports.insert(id, resource.clone());
    }
    let task_id = resource.id.clone();
    let task_format = resource.format.clone();
    let task_tenant = report_object_part(&tenant_id);
    tokio::spawn(async move {
        let started = std::time::Instant::now();
        {
            let mut exports = report_exports().write().await;
            if let Some(export) = exports.get_mut(&task_id) {
                if export.status == "cancelled" {
                    return;
                }
                export.status = "running".to_string();
                export.progress_percent = 10;
            }
        }
        let completed = async {
            let query_context = roze_report::ReportQueryContext {
                subject,
                tenant_id,
                cancellation,
            };
            let format = roze_report::ExportFormat::parse(&task_format)?;
            let rendered = roze_report::execute_export(
                report_source,
                query_context,
                task_query,
                format,
                roze_report::ExportLimits::default(),
            )
            .await?;
            let size = rendered.bytes.len() as u64;
            let key = format!("reports/{task_tenant}/{task_id}.{}", rendered.extension);
            let storage = ctx.storage()?;
            storage
                .put_object(roze_storage::PutObjectRequest {
                    key: key.clone(),
                    bytes: rendered.bytes,
                    content_type: Some(rendered.content_type),
                    metadata: BTreeMap::from([("export_id".to_string(), task_id.clone())]),
                })
                .await?;
            let download = match storage
                .presign_get(
                    &key,
                    std::time::Duration::from_secs(REPORT_EXPORT_EXPIRY_SECS),
                )
                .await
            {
                Ok(download) => download,
                Err(error) => {
                    let _ = storage.delete_object(&key).await;
                    return Err(error);
                }
            };
            Ok::<_, anyhow::Error>((key, download.url, download.expires_at_millis, size))
        }
        .await;
        let mut exports = report_exports().write().await;
        let Some(export) = exports.get_mut(&task_id) else {
            return;
        };
        if export.status == "cancelled" {
            let orphaned_key = completed.as_ref().ok().map(|(key, _, _, _)| key.clone());
            drop(exports);
            if let Some(key) = orphaned_key {
                if let Ok(storage) = ctx.storage() {
                    let _ = storage.delete_object(&key).await;
                }
            }
            return;
        }
        let mut cleanup_key = None;
        match completed {
            Ok((key, url, expires_at, size)) => {
                cleanup_key = Some(key.clone());
                export.status = "completed".to_string();
                export.progress_percent = 100;
                export.object_key = Some(key);
                export.download_url = Some(url);
                export.expires_at = Some(expires_at.to_string());
                export.terminal_at_millis = Some(report_now_millis());
                roze_metrics::record_report_export(
                    task_format.clone(),
                    "completed",
                    size,
                    started.elapsed(),
                );
                tracing::info!(event = "report.export.completed", export_id = %task_id, tenant = %task_tenant, "report export completed");
            }
            Err(_) => {
                export.status = "failed".to_string();
                export.error = Some("report export failed".to_string());
                export.terminal_at_millis = Some(report_now_millis());
                roze_metrics::record_report_export(
                    task_format.clone(),
                    "failed",
                    0,
                    started.elapsed(),
                );
                tracing::warn!(event = "report.export.failed", export_id = %task_id, tenant = %task_tenant, error_kind = "export", "report export failed");
            }
        }
        drop(exports);
        if let Some(key) = cleanup_key {
            if let Ok(storage) = ctx.storage() {
                let cleanup_id = task_id.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(REPORT_EXPORT_EXPIRY_SECS))
                        .await;
                    if storage.delete_object(&key).await.is_err() {
                        tracing::warn!(event = "report.export.cleanup_failed", export_id = %cleanup_id, object_key = %key, error_kind = "storage", "expired report object cleanup failed");
                        return;
                    }
                    report_exports().write().await.remove(&cleanup_id);
                    tracing::info!(event = "report.export.expired", export_id = %cleanup_id, object_key = %key, "expired report object removed");
                });
            }
        }
    });
    Ok(ApiResponse::ok(resource))
}

async fn report_export_status(
    State(ctx): State<ServiceContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<ApiResponse<ReportExportResource>, RozeError> {
    let (subject, tenant) = report_identity(&headers, &ctx)?;
    let export = report_exports()
        .read()
        .await
        .get(&id)
        .cloned()
        .ok_or_else(|| RozeError::NotFound(format!("report export {id}")))?;
    ensure_export_owner(&export, &subject, &tenant)?;
    Ok(ApiResponse::ok(export))
}

async fn cancel_report_export(
    State(ctx): State<ServiceContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<ApiResponse<ReportExportResource>, RozeError> {
    let (subject, tenant) = report_identity(&headers, &ctx)?;
    let mut exports = report_exports().write().await;
    let export = exports
        .get_mut(&id)
        .ok_or_else(|| RozeError::NotFound(format!("report export {id}")))?;
    ensure_export_owner(export, &subject, &tenant)?;
    if matches!(export.status.as_str(), "accepted" | "running") {
        export.status = "cancelled".to_string();
        export.cancellation.cancel();
        export.terminal_at_millis = Some(report_now_millis());
        roze_metrics::record_report_export(
            export.format.clone(),
            "cancelled",
            0,
            std::time::Duration::ZERO,
        );
        tracing::info!(event = "report.export.cancelled", export_id = %id, tenant = %tenant, subject = %subject, "report export cancelled");
    }
    Ok(ApiResponse::ok(export.clone()))
}

fn validate_chart_query(query: &ChartQueryRequest) -> Result<(), RozeError> {
    if query.chart.trim().is_empty() {
        return Err(RozeError::BadRequest("chart is required".to_string()));
    }
    if query.dimensions.len() > MAX_CHART_DIMENSIONS
        || query.group_by.len() > MAX_CHART_DIMENSIONS
        || query.measures.len() > MAX_CHART_MEASURES
        || query.filters.len() > MAX_REPORT_COLUMNS
        || query.sort.len() > MAX_REPORT_COLUMNS
        || query.limit == 0
        || query.limit > MAX_QUERY_LIMIT
    {
        return Err(RozeError::BadRequest(
            "chart query exceeds configured complexity limits".to_string(),
        ));
    }
    if query.sort.iter().any(|sort| {
        sort.field.trim().is_empty() || !matches!(sort.direction.as_str(), "asc" | "desc")
    }) {
        return Err(RozeError::BadRequest("invalid chart sort".to_string()));
    }
    Ok(())
}

async fn chart_query(
    State(ctx): State<ServiceContext>,
    headers: HeaderMap,
    Json(query): Json<ChartQueryRequest>,
) -> Result<ApiResponse<ChartQueryResponse>, RozeError> {
    let started = std::time::Instant::now();
    let (subject, tenant) = report_identity(&headers, &ctx)?;
    validate_chart_query(&query)?;
    let report_source = ctx
        .report_source()
        .map_err(|error| RozeError::Unavailable(error.to_string()))?;
    let query_context = roze_report::ReportQueryContext {
        subject: subject.clone(),
        tenant_id: tenant.clone(),
        cancellation: roze_report::ReportCancellation::new(),
    };
    let data_query = roze_report::ChartDataQuery {
        chart: query.chart.clone(),
        dimensions: query.dimensions.clone(),
        measures: query.measures.clone(),
        filters: query.filters.clone(),
        group_by: query.group_by.clone(),
        sort: query
            .sort
            .iter()
            .map(|sort| roze_report::ChartDataSort {
                field: sort.field.clone(),
                direction: sort.direction.clone(),
            })
            .collect(),
        time_bucket: query.time_bucket.clone(),
        from: query.from.clone(),
        to: query.to.clone(),
        timezone: query.timezone.clone(),
        limit: query.limit,
    };
    let dataset = roze_report::execute_chart(
        report_source,
        query_context,
        data_query,
        CHART_QUERY_TIMEOUT,
    )
    .await
    .map_err(|error| {
        tracing::warn!(event = "chart.query.failed", tenant = %tenant, subject = %subject, error_kind = "query", "chart query failed");
        if error.to_string().contains("timed out") {
            RozeError::Unavailable("chart query timed out".to_string())
        } else {
            RozeError::Internal("chart query failed".to_string())
        }
    })?;
    let result_rows = dataset
        .series
        .iter()
        .map(|series| series.points.len() as u64)
        .sum::<u64>();
    let response = ChartQueryResponse {
        chart: query.chart,
        dimensions: query.dimensions,
        measures: query.measures,
        time_bucket: query.time_bucket,
        timezone: query.timezone,
        from: query.from,
        to: query.to,
        filter_count: query.filters.len(),
        scanned_rows: dataset.scanned_rows,
        result_rows,
        series: dataset
            .series
            .into_iter()
            .map(|series| ChartSeries {
                name: series.name,
                points: series
                    .points
                    .into_iter()
                    .map(|point| ChartPoint {
                        timestamp: point.timestamp,
                        value: point.value,
                        labels: point.labels,
                    })
                    .collect(),
            })
            .collect(),
    };
    roze_metrics::record_chart_query(
        "completed",
        response.scanned_rows,
        response.result_rows,
        started.elapsed(),
    );
    tracing::info!(event = "chart.query.completed", tenant = %tenant, subject = %subject, scanned_rows = response.scanned_rows, result_rows = response.result_rows, "chart query completed");
    Ok(ApiResponse::ok(response))
}
