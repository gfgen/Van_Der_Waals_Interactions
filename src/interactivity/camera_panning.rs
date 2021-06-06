use bevy::input::mouse::MouseButtonInput;
use bevy::input::mouse::MouseMotion;

// handles user input
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;

#[derive(Default)]
pub struct Dragging(bool);

pub fn panning(
    mut pan: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<PerspectiveProjection>>,
    dragging: Res<Dragging>
) {

    for motion in pan.iter() {
        if !dragging.0 { continue; }
        let speed = 0.0005;
        let x_mov = motion.delta[0] * speed;
        let y_mov = motion.delta[1] * speed;
        let mut trans = query.single_mut()
            .expect("There should be only one camera!");

        trans.rotate(Quat::from_rotation_ypr(x_mov, y_mov, 0.0))
    }
}

pub fn detect_dragging(
    mut clicks: EventReader<MouseButtonInput>,
    mut dragging: ResMut<Dragging>
) {
    for click in clicks.iter() {
        if click.button == MouseButton::Left { dragging.0 = click.state.is_pressed(); }
    }
}

pub struct CameraPanning;

impl Plugin for CameraPanning {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<Dragging>()
            .add_system(detect_dragging.system())
            .add_system(panning.system());
    }
}