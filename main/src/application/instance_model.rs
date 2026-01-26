use crate::application::{Affected, ChangeResult};
use crate::domain::{SharedInstance, Tag};
use anyhow::Context;
use base::spawn_in_main_thread;
use derivative::Derivative;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type SharedInstanceModel = Rc<RefCell<InstanceModel>>;
pub type WeakInstanceModel = Weak<RefCell<InstanceModel>>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct InstanceModel {
    tags: Vec<Tag>,
    instance: SharedInstance,
    #[derivative(Debug = "ignore")]
    ui: Box<dyn InstanceUi>,
}

impl InstanceModel {
    pub fn new(instance: SharedInstance, ui: Box<dyn InstanceUi>) -> Self {
        Self {
            tags: Default::default(),
            instance,
            ui,
        }
    }

    pub fn instance(&self) -> &SharedInstance {
        &self.instance
    }

    /// Returns all instance tags.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Modifies this instance by executing an instance command and returns the affected properties.
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

    /// Modifies this instance by executing an instance command and returns the affected properties.
    ///
    /// Invokes listeners.
    pub fn change_with_notification(
        &mut self,
        cmd: InstanceCommand,
        initiator: Option<u32>,
        weak_model: WeakInstanceModel,
    ) {
        if let Ok(Some(affected)) = self.change(cmd) {
            spawn_in_main_thread(async move {
                let model = weak_model.upgrade().context("upgrading model")?;
                model.borrow().ui.handle_affected(affected, initiator)?;
                Ok(())
            });
        }
    }
}

pub trait InstanceUi {
    fn handle_affected(
        &self,
        affected: Affected<InstanceProp>,
        initiator: Option<u32>,
    ) -> anyhow::Result<()>;
}

pub enum InstanceCommand {
    SetTags(Vec<Tag>),
}

pub enum InstanceProp {
    Tags,
}
