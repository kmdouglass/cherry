pub mod rays;
pub mod sequential_model;
pub mod trace;

trait SystemModel {
    fn gaps(&self) -> &[sequential_model::Gap];
    fn surfaces(&self) -> &[sequential_model::Surface];
}