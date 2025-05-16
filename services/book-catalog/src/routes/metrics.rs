use actix_web::{web::Data, HttpResponse, Responder};
use metrics_exporter_prometheus::PrometheusHandle;

pub async fn get_metrics(handle: Data<PrometheusHandle>) -> impl Responder {
    let metrics = handle.render();
    HttpResponse::Ok().content_type("text/plain").body(metrics)
}