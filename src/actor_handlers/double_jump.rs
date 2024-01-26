use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DoubleJumpHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for DoubleJumpHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let attributes = actor.attributes.borrow();

        if let Some(Attribute::ActiveActor(active_actor)) =
            attributes.get("TAGame.CarComponent_TA:Vehicle")
        {
            let car_actor_id = active_actor.actor;
            let car_ids_to_player_ids = self.frame_parser.car_ids_to_player_ids.borrow();
            if let Some(player_actor_id) = car_ids_to_player_ids.get(&car_actor_id) {
                let double_jump_data = TimeSeriesDoubleJumpData::from(actor);
                let mut players_data = self
                    .frame_parser
                    .players_time_series_double_jump_data
                    .borrow_mut();

                let players_wrapped_unique_id =
                    self.frame_parser.players_wrapped_unique_id.borrow();
                let player_wrapped_unique_id =
                    players_wrapped_unique_id.get(player_actor_id).unwrap();
                match players_data.get_mut(player_wrapped_unique_id) {
                    Some(player_data) => {
                        player_data.insert(frame_number, double_jump_data);
                    }
                    None => {
                        let mut player_data =
                            HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                        player_data.insert(frame_number, double_jump_data);
                        players_data.insert(player_wrapped_unique_id.clone(), player_data);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesDoubleJumpData {
    pub double_jump_is_active: Option<bool>,
    pub double_jump_torque_x: Option<f32>,
    pub double_jump_torque_y: Option<f32>,
    pub double_jump_torque_z: Option<f32>,
}

impl TimeSeriesDoubleJumpData{
    pub fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();

        let mut double_jump_is_active = None;
        if let Some(Attribute::Byte(_double_jump_is_active_int)) =
            attributes.get("TAGame.CarComponent_TA:ReplicatedActive")
        {
            double_jump_is_active = Some(*_double_jump_is_active_int & 1 != 0); // active when the integer is odd.
        }
        let double_jump_torque_x = attributes
            .get("TAGame.CarComponent_Dodge_TA:DodgeTorque")
            .and_then(|attr| match attr {
                Attribute::Location(double_jump_torque) => {
                    Some(double_jump_torque.x)
                }
                _ => None,
            });
        let double_jump_torque_y = attributes
            .get("TAGame.CarComponent_Dodge_TA:DodgeTorque")
            .and_then(|attr| match attr {
                Attribute::Location(double_jump_torque) => {
                    Some(double_jump_torque.y)
                }
                _ => None,
            });
        let double_jump_torque_z = attributes
            .get("TAGame.CarComponent_Dodge_TA:DodgeTorque")
            .and_then(|attr| match attr {
                Attribute::Location(double_jump_torque) => {
                    Some(double_jump_torque.z)
                }
                _ => None,
            });
        TimeSeriesDoubleJumpData {
            double_jump_is_active,
            double_jump_torque_x,
            double_jump_torque_y,
            double_jump_torque_z,
        }
    }
}