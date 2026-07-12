use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AirActivateHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for AirActivateHandler<'a> {
    fn new(frame_parser: &'a FrameParser) -> Self {
        Self { frame_parser }
    }

    fn update(&mut self, actor: &Actor, frame_number: usize, _time: f32, _delta: f32) {
        let attributes = actor.attributes.borrow();

        // The active actor linking this component to a vehicle
        if let Some(Attribute::ActiveActor(active_actor)) =
            attributes.get("TAGame.CarComponent_TA:Vehicle")
        {
            let car_actor_id = active_actor.actor;
            let car_ids_to_player_ids = self.frame_parser.car_ids_to_player_ids.borrow();

            if let Some(player_actor_id) = car_ids_to_player_ids.get(&car_actor_id) {
                let air_activate_data = TimeSeriesAirActivateData::from(actor);

                // Note: You must add `players_time_series_air_activate_data` to FrameParser
                let mut players_data = self
                    .frame_parser
                    .players_time_series_air_activate_data
                    .borrow_mut();

                let players_wrapped_unique_id =
                    self.frame_parser.players_wrapped_unique_id.borrow();
                let player_wrapped_unique_id =
                    players_wrapped_unique_id.get(player_actor_id).unwrap();

                match players_data.get_mut(player_wrapped_unique_id) {
                    Some(player_data) => {
                        player_data.insert(frame_number, air_activate_data);
                    }
                    None => {
                        let mut player_data =
                            HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                        player_data.insert(frame_number, air_activate_data);
                        players_data.insert(player_wrapped_unique_id.clone(), player_data);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesAirActivateData {
    pub air_activate_count: Option<u32>, // Adjust type based on Boxcars output (e.g., u8 for Byte)
}

impl TimeSeriesAirActivateData {
    pub fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();
        let mut air_activate_count = None;

        if let Some(Attribute::Int(_count)) =
            attributes.get("TAGame.CarComponent_AirActivate_TA:AirActivateCount")
        {
            air_activate_count = Some(*_count as u32);
        }

        TimeSeriesAirActivateData { air_activate_count }
    }
}