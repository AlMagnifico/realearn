use crate::infrastructure::plugin::App;
use playtime_clip_engine::base::Matrix;
use playtime_clip_engine::proto::clip_engine_server::ClipEngineServer;
use playtime_clip_engine::proto::{ClipEngineHub, ClipEngineService, MatrixProvider};

pub type PlaytimeService = ClipEngineServer<ClipEngineService<AppMatrixProvider>>;

pub fn create_playtime_service(hub: &ClipEngineHub) -> PlaytimeService {
    hub.create_service(AppMatrixProvider)
}

pub struct AppMatrixProvider;

impl MatrixProvider for AppMatrixProvider {
    fn with_matrix<R>(
        &self,
        clip_matrix_id: &str,
        f: impl FnOnce(&Matrix) -> R,
    ) -> Result<R, &'static str> {
        App::get().with_clip_matrix(clip_matrix_id, f)
    }

    fn with_matrix_mut<R>(
        &self,
        clip_matrix_id: &str,
        f: impl FnOnce(&mut Matrix) -> R,
    ) -> Result<R, &'static str> {
        App::get().with_clip_matrix_mut(clip_matrix_id, f)
    }
}
