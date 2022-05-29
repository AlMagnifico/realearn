use crate::infrastructure::plugin::App;
use crate::infrastructure::server::grpc::WithSessionId;
use futures::{Stream, StreamExt};
use playtime_clip_engine::proto::{
    clip_engine_server, GetContinuousMatrixUpdatesReply, GetContinuousMatrixUpdatesRequest,
    GetContinuousSlotUpdatesReply, GetContinuousSlotUpdatesRequest, GetContinuousTrackUpdatesReply,
    GetContinuousTrackUpdatesRequest, GetOccasionalSlotUpdatesReply,
    GetOccasionalSlotUpdatesRequest,
};
use std::pin::Pin;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct RealearnClipEngine {}

#[tonic::async_trait]
impl clip_engine_server::ClipEngine for RealearnClipEngine {
    type GetContinuousMatrixUpdatesStream =
        SyncBoxStream<'static, Result<GetContinuousMatrixUpdatesReply, Status>>;
    type GetContinuousTrackUpdatesStream =
        SyncBoxStream<'static, Result<GetContinuousTrackUpdatesReply, Status>>;
    type GetContinuousSlotUpdatesStream =
        SyncBoxStream<'static, Result<GetContinuousSlotUpdatesReply, Status>>;
    type GetOccasionalSlotUpdatesStream =
        SyncBoxStream<'static, Result<GetOccasionalSlotUpdatesReply, Status>>;

    async fn get_continuous_slot_updates(
        &self,
        request: Request<GetContinuousSlotUpdatesRequest>,
    ) -> Result<Response<Self::GetContinuousSlotUpdatesStream>, Status> {
        let receiver = App::get().continuous_slot_update_sender().subscribe();
        let requested_clip_matrix_id = request.into_inner().clip_matrix_id;
        let receiver_stream = BroadcastStream::new(receiver).filter_map(move |value| {
            // TODO-high This shouldn't be necessary!
            let requested_clip_matrix_id = requested_clip_matrix_id.clone();
            async move {
                match value {
                    Err(e) => Some(Err(Status::unknown(e.to_string()))),
                    Ok(WithSessionId { session_id, value })
                        if &session_id == &requested_clip_matrix_id =>
                    {
                        Some(Ok(GetContinuousSlotUpdatesReply {
                            slot_updates: value,
                        }))
                    }
                    _ => None,
                }
            }
        });
        Ok(Response::new(Box::pin(receiver_stream)))
    }

    async fn get_continuous_matrix_updates(
        &self,
        request: Request<GetContinuousMatrixUpdatesRequest>,
    ) -> Result<Response<Self::GetContinuousMatrixUpdatesStream>, Status> {
        todo!()
    }

    async fn get_continuous_track_updates(
        &self,
        request: Request<GetContinuousTrackUpdatesRequest>,
    ) -> Result<Response<Self::GetContinuousTrackUpdatesStream>, Status> {
        todo!()
    }

    async fn get_occasional_slot_updates(
        &self,
        request: Request<GetOccasionalSlotUpdatesRequest>,
    ) -> Result<Response<Self::GetOccasionalSlotUpdatesStream>, Status> {
        todo!()
    }
}

type SyncBoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + Sync + 'a>>;
