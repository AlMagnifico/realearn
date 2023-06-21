use crate::conversion_util::adjust_proportionally_positive;
use crate::rt::buffer::AudioBufMut;
use crate::rt::supplier::{
    AudioSupplier, AutoDelegatingPositionTranslationSkill, AutoDelegatingPreBufferSourceSkill,
    AutoDelegatingWithMaterialInfo, SupplyAudioRequest,
    SupplyResponse, SupplyResponseStatus, WithMaterialInfo, WithSupplier, MIDI_FRAME_RATE,
};
use crate::rt::supplier::{
    MidiSupplier, SupplyMidiRequest, SupplyRequestInfo,
};

use playtime_api::persistence::VirtualResampleMode;
use reaper_high::Reaper;
use reaper_low::raw;
use reaper_medium::{BorrowedMidiEventList, Hz, OwnedReaperResample};
use std::ffi::c_void;
use std::ptr::null_mut;

#[derive(Debug)]
pub struct Resampler<S> {
    /// If enabled in general.
    enabled: bool,
    tempo_adjustments_enabled: bool,
    responsible_for_audio_tempo_adjustments: bool,
    supplier: S,
    api: OwnedReaperResample,
    tempo_factor: f64,
}

impl<S> WithSupplier for Resampler<S> {
    type Supplier = S;

    fn supplier(&self) -> &Self::Supplier {
        &self.supplier
    }

    fn supplier_mut(&mut self) -> &mut Self::Supplier {
        &mut self.supplier
    }
}

impl<S> Resampler<S> {
    pub fn new(supplier: S) -> Self {
        let api = Reaper::get().medium_reaper().resampler_create();
        Self {
            enabled: false,
            tempo_adjustments_enabled: false,
            responsible_for_audio_tempo_adjustments: false,
            supplier,
            api,
            tempo_factor: 1.0,
        }
    }

    pub fn reset_buffers_and_latency(&mut self) {
        self.api.as_mut().as_mut().Reset();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_mode(&mut self, mode: VirtualResampleMode) {
        use VirtualResampleMode::*;
        let raw_mode = match mode {
            ProjectDefault => -1,
            ReaperMode(m) => m.mode as i32,
        };
        unsafe {
            self.api.as_mut().as_mut().Extended(
                raw::RESAMPLE_EXT_SETRSMODE,
                raw_mode as *const c_void as *mut _,
                null_mut(),
                null_mut(),
            );
        }
    }

    /// If the part of the resampler is enabled that modifies the tempo of the material.
    pub fn set_tempo_adjustments_enabled(&mut self, enabled: bool) {
        self.tempo_adjustments_enabled = enabled;
    }

    /// Decides whether the resampler should also take the tempo factor into account for audio
    /// (VariSpeed).
    ///
    /// Usually, it only takes care of adjusting the MIDI tempo.
    ///
    /// This only has an effect if tempo adjustments are enabled in general.
    pub fn set_responsible_for_audio_tempo_adjustments(&mut self, responsible: bool) {
        self.responsible_for_audio_tempo_adjustments = responsible;
    }

    /// Only has an effect if tempo changing enabled.
    pub fn set_tempo_factor(&mut self, tempo_factor: f64) {
        self.tempo_factor = tempo_factor;
    }
}

impl<S: AudioSupplier + WithMaterialInfo> AudioSupplier for Resampler<S> {
    fn supply_audio(
        &mut self,
        request: &SupplyAudioRequest,
        dest_buffer: &mut AudioBufMut,
    ) -> SupplyResponse {
        if !self.enabled {
            return self.supplier.supply_audio(request, dest_buffer);
        }
        let material_info = self.supplier.material_info().unwrap();
        let source_frame_rate = material_info.frame_rate();
        let dest_frame_rate = request.dest_sample_rate.unwrap_or(source_frame_rate);
        let dest_frame_rate =
            if self.tempo_adjustments_enabled && self.responsible_for_audio_tempo_adjustments {
                Hz::new(dest_frame_rate.get() / self.tempo_factor)
            } else {
                dest_frame_rate
            };
        if source_frame_rate == dest_frame_rate {
            return self.supplier.supply_audio(request, dest_buffer);
        }
        let mut total_num_frames_consumed = 0usize;
        let mut total_num_frames_written = 0usize;
        let source_channel_count = material_info.channel_count();
        let api = self.api.as_mut().as_mut();
        api.SetRates(source_frame_rate.get(), dest_frame_rate.get());
        // Set ResamplePrepare's out_samples to refer to request a specific number of input samples.
        // const RESAMPLE_EXT_SETFEEDMODE: i32 = 0x1001;
        // let ext_result = unsafe {
        //     self.mode.api.Extended(
        //         RESAMPLE_EXT_SETFEEDMODE,
        //         1 as *mut _,
        //         null_mut(),
        //         null_mut(),
        //     )
        // };
        let reached_end = loop {
            // Get resampler buffer.
            let buffer_frame_count = 128usize;
            let mut resample_buffer: *mut f64 = null_mut();
            let num_source_frames_to_write = unsafe {
                api.ResamplePrepare(
                    buffer_frame_count as _,
                    source_channel_count as i32,
                    &mut resample_buffer,
                )
            };
            if num_source_frames_to_write == 0 {
                // We are probably responsible for tempo adjustment and the tempo is super low.
                break false;
            }
            let mut resample_buffer = unsafe {
                AudioBufMut::from_raw(
                    resample_buffer,
                    source_channel_count,
                    num_source_frames_to_write as _,
                )
            };
            // Feed resampler buffer with source material.
            let inner_request = SupplyAudioRequest {
                start_frame: request.start_frame + total_num_frames_consumed as isize,
                dest_sample_rate: None,
                info: SupplyRequestInfo {
                    audio_block_frame_offset: request.info.audio_block_frame_offset
                        + total_num_frames_written,
                    requester: "resampler-audio",
                    note: "",
                    is_realtime: false,
                },
                parent_request: Some(request),
                general_info: request.general_info,
            };
            let inner_response = self
                .supplier
                .supply_audio(&inner_request, &mut resample_buffer);
            if inner_response.status.reached_end() {
                break true;
            }
            total_num_frames_consumed += inner_response.num_frames_consumed;
            // Get output material.
            let offset_buffer = dest_buffer.slice_mut(total_num_frames_written..);
            let num_frames_written = unsafe {
                api.ResampleOut(
                    offset_buffer.data_as_mut_ptr(),
                    num_source_frames_to_write,
                    offset_buffer.frame_count() as _,
                    source_channel_count as _,
                )
            };
            total_num_frames_written += num_frames_written as usize;
            if total_num_frames_written >= dest_buffer.frame_count() {
                // We have enough resampled material.
                break false;
            }
        };
        SupplyResponse {
            num_frames_consumed: total_num_frames_consumed,
            status: if reached_end {
                SupplyResponseStatus::ReachedEnd {
                    num_frames_written: total_num_frames_written,
                }
            } else {
                SupplyResponseStatus::PleaseContinue
            },
        }
    }
}

impl<S: MidiSupplier> MidiSupplier for Resampler<S> {
    fn supply_midi(
        &mut self,
        request: &SupplyMidiRequest,
        event_list: &mut BorrowedMidiEventList,
    ) -> SupplyResponse {
        if !self.enabled {
            return self.supplier.supply_midi(request, event_list);
        }
        let source_frame_rate = MIDI_FRAME_RATE;
        if request.dest_sample_rate == source_frame_rate {
            // Should never be the case because we have an artificial fixed MIDI frame rate that
            // is unlike any realistic sample rate.
            return self.supplier.supply_midi(request, event_list);
        }
        let num_frames_to_be_written = request.dest_frame_count;
        let request_ratio = num_frames_to_be_written as f64 / request.dest_sample_rate.get();
        let final_ratio = if self.tempo_adjustments_enabled {
            request_ratio * self.tempo_factor
        } else {
            request_ratio
        };
        let num_frames_to_be_consumed =
            adjust_proportionally_positive(source_frame_rate.get(), final_ratio);
        let inner_request = SupplyMidiRequest {
            start_frame: request.start_frame,
            dest_frame_count: num_frames_to_be_consumed,
            dest_sample_rate: source_frame_rate,
            info: SupplyRequestInfo {
                audio_block_frame_offset: request.info.audio_block_frame_offset,
                requester: "resampler-midi",
                note: "",
                is_realtime: true,
            },
            parent_request: Some(request),
            general_info: request.general_info,
        };
        let inner_response = self.supplier.supply_midi(&inner_request, event_list);
        SupplyResponse {
            num_frames_consumed: inner_response.num_frames_consumed,
            status: {
                use SupplyResponseStatus::*;
                match inner_response.status {
                    PleaseContinue => PleaseContinue,
                    ReachedEnd { num_frames_written } => {
                        let response_ratio =
                            num_frames_to_be_written as f64 / num_frames_to_be_consumed as f64;
                        ReachedEnd {
                            num_frames_written: adjust_proportionally_positive(
                                num_frames_written as f64,
                                response_ratio,
                            ),
                        }
                    }
                }
            },
        }
    }
}

impl<S> AutoDelegatingWithMaterialInfo for Resampler<S> {}
impl<S> AutoDelegatingPreBufferSourceSkill for Resampler<S> {}

// There's no translation because the resampler doesn't actually change the scale
// in which positions are measured, not even if the resampler is responsible for
// tempo adjustments. E.g. if the tempo is higher, the play position will just do larger
// steps forward.
impl<S> AutoDelegatingPositionTranslationSkill for Resampler<S> {}
