use std::sync::Arc;

use hecs::{Entity, World};
use hv_cell::AtomicRefCell;
use hv_lua::UserData;
use tealr::{mlu::TealData, TypeBody, TypeName};

pub type SystemFn = fn(Arc<AtomicRefCell<World>>);

/// This is used as a component to signify ownership
#[derive(Clone, TypeName)]
pub struct Owner(pub Entity);

impl UserData for Owner {}
impl TealData for Owner {}
impl TypeBody for Owner {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
    }
}
/// Placeholder until we implement threading
#[derive(Default)]
pub struct SchedulerBuilder {
    steps: Vec<SystemFn>,
}

impl SchedulerBuilder {
    #[must_use]
    pub fn with_system(self, system: SystemFn) -> Self {
        let mut steps = self.steps;
        steps.push(system);

        SchedulerBuilder { steps }
    }

    pub fn add_system(&mut self, system: SystemFn) -> &mut Self {
        self.steps.push(system);
        self
    }

    #[must_use]
    pub fn with_thread_local(self, system: SystemFn) -> Self {
        let mut steps = self.steps;
        steps.push(system);

        SchedulerBuilder { steps }
    }

    pub fn add_thread_local(&mut self, system: SystemFn) -> &mut Self {
        self.steps.push(system);
        self
    }

    pub fn build(self) -> Scheduler {
        Scheduler { steps: self.steps }
    }
}

/// Placeholder until we implement threading
pub struct Scheduler {
    steps: Vec<SystemFn>,
}

impl Scheduler {
    pub fn builder() -> SchedulerBuilder {
        SchedulerBuilder::default()
    }

    pub fn execute(&mut self, world: Arc<AtomicRefCell<World>>) {
        for f in &mut self.steps {
            f(world.clone());
        }
    }
}
