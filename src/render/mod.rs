pub mod json;
pub mod tree;

use crate::model::*;

pub trait Render {
    fn today(&self, items: &[SrItem]);
    fn stats(&self, stats: &SrStats);
    fn review(&self, result: &ReviewResult);
    fn init_sr(&self, result: &InitResult);
    fn query(&self, result: &QueryResult);
    fn enforce(&self, result: &EnforceResult);
}

pub fn get(is_json: bool) -> Box<dyn Render> {
    if is_json {
        Box::new(json::JsonRenderer)
    } else {
        Box::new(tree::TreeRenderer)
    }
}
