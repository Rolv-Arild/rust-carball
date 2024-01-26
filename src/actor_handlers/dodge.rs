use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DodgeHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for DodgeHandler<'a> {
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
                let dodge_data = TimeSeriesDodgeData::from(actor);
                let mut players_data = self
                    .frame_parser
                    .players_time_series_dodge_data
                    .borrow_mut();

                let players_wrapped_unique_id =
                    self.frame_parser.players_wrapped_unique_id.borrow();
                let player_wrapped_unique_id =
                    players_wrapped_unique_id.get(player_actor_id).unwrap();
                match players_data.get_mut(player_wrapped_unique_id) {
                    Some(player_data) => {
                        player_data.insert(frame_number, dodge_data);
                    }
                    None => {
                        let mut player_data =
                            HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                        player_data.insert(frame_number, dodge_data);
                        players_data.insert(player_wrapped_unique_id.clone(), player_data);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesDodgeData {
    pub dodge_is_active: Option<bool>,
    pub dodge_torque_x: Option<f32>,
    pub dodge_torque_y: Option<f32>,
    pub dodge_torque_z: Option<f32>,
}

impl TimeSeriesDodgeData{
    pub fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();

        let mut dodge_is_active = None;
        if let Some(Attribute::Byte(_dodge_is_active_int)) =
            attributes.get("TAGame.CarComponent_TA:ReplicatedActive")
        {
            dodge_is_active = Some(*_dodge_is_active_int & 1 != 0); // active when the integer is odd.
        }
        let dodge_torque_x = attributes
            .get("TAGame.CarComponent_Dodge_TA:DodgeTorque")
            .and_then(|attr| match attr {
                Attribute::Location(dodge_torque) => {
                    Some(dodge_torque.x)
                }
                _ => None,
            });
        let dodge_torque_y = attributes
            .get("TAGame.CarComponent_Dodge_TA:DodgeTorque")
            .and_then(|attr| match attr {
                Attribute::Location(dodge_torque) => {
                    Some(dodge_torque.y)
                }
                _ => None,
            });     
        let dodge_torque_z = attributes
            .get("TAGame.CarComponent_Dodge_TA:DodgeTorque")
            .and_then(|attr| match attr {
                Attribute::Location(dodge_torque) => {
                    Some(dodge_torque.z)
                }
                _ => None,
            });       
        TimeSeriesDodgeData {
            dodge_is_active,
            dodge_torque_x,
            dodge_torque_y,
            dodge_torque_z
        }
    }
}