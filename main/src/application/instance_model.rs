use crate::application::{Affected, ChangeResult};
use crate::domain::{SharedInstance, Tag};
use std::cell::RefCell;
use std::rc::Rc;

pub type SharedInstanceModel = Rc<RefCell<InstanceModel>>;

#[derive(Debug)]
pub struct InstanceModel {
    tags: Vec<Tag>,
    instance: SharedInstance,
}

impl InstanceModel {
    pub fn new(instance: SharedInstance) -> Self {
        Self {
            tags: Default::default(),
            instance,
        }
    }

    pub fn instance(&self) -> &SharedInstance {
        &self.instance
    }

    /// Returns all instance tags.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
    /// Executes an instance command and returns the affected properties.
    ///
    /// Doesn't invoke listeners.
    pub fn change(&mut self, cmd: InstanceCommand) -> ChangeResult<InstanceProp> {
        let affected = match cmd {
            InstanceCommand::SetTags(tags) => {
                self.tags = tags;
                Some(Affected::One(InstanceProp::Tags))
            }
        };
        Ok(affected)
    }
}

pub enum InstanceCommand {
    SetTags(Vec<Tag>),
}

pub enum InstanceProp {
    Tags,
}
