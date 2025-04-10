use bevy::prelude::*;
use bevy_map_viewer::ZoomChangedEvent;

use crate::types::MapBundle;

use super::{OverpassClient, OverpassReceiver, OverpassWorkerPlugin};

pub struct OverpassPlugin;
// TODO: Fix requests for routes, relations and points. THESE ARE A MATTER OF PRIORITY!
impl Plugin for OverpassPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapBundle::new())
            .insert_resource(OverpassClientResource::default())
            .add_plugins(OverpassWorkerPlugin)
            .add_systems(FixedUpdate, read_overpass_receiver);
    }
}

#[derive(Resource, Default, Clone)]
pub struct OverpassClientResource {
    pub client: OverpassClient,
}

pub fn read_overpass_receiver(
    map_receiver: Option<Res<OverpassReceiver>>,
    mut map_bundle: ResMut<MapBundle>,
    mut zoom_event: EventWriter<ZoomChangedEvent>,
) {
    if let Some(map_receiver) = map_receiver {
        if let Ok(v) = map_receiver.0.try_recv() {
            for feature in &v {
                map_bundle.features.insert(feature.clone());
            }
            zoom_event.send(ZoomChangedEvent);
        }
    }
}
