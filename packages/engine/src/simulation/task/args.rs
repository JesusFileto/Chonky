use crate::simulation::enum_dispatch::*;

use crate::config::TaskDistributionConfig;
use crate::simulation::packages::init::packages::jspy::js::JsInitTask;
use crate::simulation::packages::init::packages::jspy::py::PyInitTask;

#[enum_dispatch]
pub trait GetTaskArgs {
    fn distribution(&self) -> TaskDistributionConfig;
}
