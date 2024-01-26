use crate::actor_handlers::ActorHandler;
use crate::frame_parser::{Actor, FrameParser};
use boxcars::Attribute;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FlipCarHandler<'a> {
    frame_parser: &'a FrameParser,
}

impl<'a> ActorHandler<'a> for FlipCarHandler<'a> {
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
                let flip_car_data = TimeSeriesFlipCarData::from(actor);
                let mut players_data = self
                    .frame_parser
                    .players_time_series_flip_car_data
                    .borrow_mut();

                let players_wrapped_unique_id =
                    self.frame_parser.players_wrapped_unique_id.borrow();
                let player_wrapped_unique_id =
                    players_wrapped_unique_id.get(player_actor_id).unwrap();
                match players_data.get_mut(player_wrapped_unique_id) {
                    Some(player_data) => {
                        player_data.insert(frame_number, flip_car_data);
                    }
                    None => {
                        let mut player_data =
                            HashMap::with_capacity(self.frame_parser.frame_count - frame_number);
                        player_data.insert(frame_number, flip_car_data);
                        players_data.insert(player_wrapped_unique_id.clone(), player_data);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSeriesFlipCarData {
    pub flip_car_is_active: Option<bool>,
}

impl TimeSeriesFlipCarData {
    pub fn from(actor: &Actor) -> Self {
        let attributes = actor.attributes.borrow();

        let mut flip_car_is_active = None;
        if let Some(Attribute::Byte(_flip_car_is_active_int)) =
            attributes.get("TAGame.CarComponent_TA:ReplicatedActive")
        {
            flip_car_is_active = Some(*_flip_car_is_active_int & 1 != 0); // Jump is active when the integer is odd.
        }
        TimeSeriesFlipCarData {
            flip_car_is_active,
        }
    }
}