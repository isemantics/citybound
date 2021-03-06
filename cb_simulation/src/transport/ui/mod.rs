use descartes::{LinePath, Segment, WithUniqueOrthogonal};
use compact::CVec;
use kay::{ActorSystem, World, TypedID, RawID};
use michelangelo::{Instance, Mesh};
use super::lane::{Lane, LaneID, SwitchLane, SwitchLaneID};

use dimensions::{LANE_DISTANCE, LANE_WIDTH, LANE_MARKER_WIDTH, LANE_MARKER_DASH_GAP,
LANE_MARKER_DASH_LENGTH};

use itertools::Itertools;

pub trait TransportUI {
    fn on_lane_constructed(
        &mut self,
        id: RawID,
        lane_path: &LinePath,
        is_switch: bool,
        on_intersection: bool,
        _world: &mut World,
    );

    fn on_lane_destructed(
        &mut self,
        id: RawID,
        is_switch: bool,
        on_intersection: bool,
        _world: &mut World,
    );
    fn on_car_instances(&mut self, from_lane: RawID, instances: &CVec<Instance>, _: &mut World);
}

impl Lane {
    fn car_instances(&self) -> CVec<Instance> {
        let mut cars_iter = self.microtraffic.cars.iter();
        let mut car_instances = CVec::with_capacity(self.microtraffic.cars.len());
        for (segment, distance_pair) in self.construction.path.segments_with_distances() {
            for car in
                cars_iter.take_while_ref(|car| *car.position - distance_pair[0] < segment.length())
            {
                let position2d = segment.along(*car.position - distance_pair[0]);
                let direction = segment.direction();
                car_instances.push(Instance {
                    instance_position: [position2d.x, position2d.y, 0.0],
                    instance_direction: [direction.x, direction.y],
                    instance_color: [0.0, 0.0, 0.0],
                })
            }
        }

        car_instances
    }

    pub fn get_car_instances(&self, ui: TransportUIID, world: &mut World) {
        ui.on_car_instances(self.id.as_raw(), self.car_instances(), world);
    }
}

pub fn lane_mesh(path: &LinePath) -> Mesh {
    Mesh::from_path_as_band(path, LANE_WIDTH, 0.0)
}

pub fn marker_mesh(path: &LinePath) -> (Mesh, Mesh) {
    // use negative widths to simulate a shifted band on each side
    (
        Mesh::from_path_as_band_asymmetric(
            &path,
            LANE_DISTANCE / 2.0 + LANE_MARKER_WIDTH / 2.0,
            -(LANE_DISTANCE / 2.0 - LANE_MARKER_WIDTH / 2.0),
            0.1,
        ),
        Mesh::from_path_as_band_asymmetric(
            &path,
            -(LANE_DISTANCE / 2.0 - LANE_MARKER_WIDTH / 2.0),
            LANE_DISTANCE / 2.0 + LANE_MARKER_WIDTH / 2.0,
            0.1,
        ),
    )
}

pub fn switch_marker_gap_mesh(path: &LinePath) -> Mesh {
    path.dash(LANE_MARKER_DASH_GAP, LANE_MARKER_DASH_LENGTH)
        .into_iter()
        .filter_map(|maybe_dash| {
            maybe_dash.map(|dash| Mesh::from_path_as_band(&dash, LANE_MARKER_WIDTH * 2.0, 0.0))
        })
        .sum()
}

impl Lane {
    pub fn get_render_info(&mut self, ui: TransportUIID, world: &mut World) {
        ui.on_lane_constructed(
            self.id.as_raw(),
            self.construction.path.clone(),
            false,
            self.connectivity.on_intersection,
            world,
        );
    }
}

impl SwitchLane {
    pub fn get_render_info(&mut self, ui: TransportUIID, world: &mut World) {
        ui.on_lane_constructed(
            self.id.as_raw(),
            self.construction.path.clone(),
            true,
            false,
            world,
        );
    }
}

impl SwitchLane {
    fn car_instances(&self) -> CVec<Instance> {
        let mut cars_iter = self.microtraffic.cars.iter();
        let mut car_instances = CVec::with_capacity(self.microtraffic.cars.len());
        for (segment, distance_pair) in self.construction.path.segments_with_distances() {
            for car in
                cars_iter.take_while_ref(|car| *car.position - distance_pair[0] < segment.length())
            {
                let position2d = segment.along(*car.position - distance_pair[0]);
                let direction = segment.direction();
                let rotated_direction = (direction
                    + 0.3 * car.switch_velocity * direction.orthogonal_right())
                .normalize();
                let shifted_position2d =
                    position2d + 2.5 * direction.orthogonal_right() * car.switch_position;
                car_instances.push(Instance {
                    instance_position: [shifted_position2d.x, shifted_position2d.y, 0.0],
                    instance_direction: [rotated_direction.x, rotated_direction.y],
                    instance_color: [0.0, 0.0, 0.0],
                })
            }
        }

        car_instances
    }

    pub fn get_car_instances(&mut self, ui: TransportUIID, world: &mut World) {
        ui.on_car_instances(self.id.as_raw(), self.car_instances(), world);
    }
}

pub fn on_build(lane: &Lane, world: &mut World) {
    TransportUIID::global_broadcast(world).on_lane_constructed(
        lane.id.as_raw(),
        lane.construction.path.clone(),
        false,
        lane.connectivity.on_intersection,
        world,
    );
}

pub fn on_build_switch(lane: &SwitchLane, world: &mut World) {
    TransportUIID::global_broadcast(world).on_lane_constructed(
        lane.id.as_raw(),
        lane.construction.path.clone(),
        true,
        false,
        world,
    );
}

pub fn on_unbuild(lane: &Lane, world: &mut World) {
    TransportUIID::global_broadcast(world).on_lane_destructed(
        lane.id.as_raw(),
        false,
        lane.connectivity.on_intersection,
        world,
    );
}

pub fn on_unbuild_switch(lane: &SwitchLane, world: &mut World) {
    TransportUIID::global_broadcast(world).on_lane_destructed(lane.id.as_raw(), true, false, world);
}

pub fn setup(system: &mut ActorSystem) {
    auto_setup(system);
}

mod kay_auto;
pub use self::kay_auto::*;
