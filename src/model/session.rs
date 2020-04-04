use crate::model::MidiSourceModel;

#[derive(Default)]
pub struct RealearnSession<'a> {
    dummy_source_model: MidiSourceModel<'a>,
}

impl<'a> RealearnSession<'a> {
    pub fn new() -> RealearnSession<'a> {
        RealearnSession::default()
    }

    pub fn get_dummy_source_model(&self) -> &MidiSourceModel<'a> {
        &self.dummy_source_model
    }
}
