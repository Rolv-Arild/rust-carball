use crate::actor_handlers::{TeamData, WrappedUniqueId};
use crate::frame_parser::FrameParser;
use boxcars::attributes::RemoteId;
use boxcars::{ActorId, Attribute};
use log::{debug, error, warn};
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Player {
    #[serde(serialize_with = "serialize_wrapped_unique_id")]
    pub unique_id: WrappedUniqueId,
    pub name: String,
    pub online_id: Option<String>,
    pub online_id_kind: Option<String>,
    pub is_orange: Option<bool>,
    pub car_id: Option<i32>,
    pub match_score: i32,
    pub match_goals: i32,
    pub match_assists: i32,
    pub match_saves: i32,
    pub match_shots: i32,
}

impl Player {
    pub fn from_frame_parser(frame_parser: &FrameParser) -> Vec<Self> {
        let players_actor = frame_parser.players_actor.borrow();
        let teams_actor = frame_parser.teams_data.borrow();
        let players_teams = frame_parser.players_teams.borrow();
        players_actor
            .iter()
            .map(|(wrapped_unique_id, player_actor)| {
                Player::from(
                    wrapped_unique_id,
                    player_actor,
                    &teams_actor,
                    &players_teams,
                )
            })
            .collect()
    }

    pub fn from(
        wrapped_unique_id: &WrappedUniqueId,
        attributes: &HashMap<String, boxcars::Attribute>,
        teams_actor: &HashMap<ActorId, TeamData>,
        players_teams: &HashMap<WrappedUniqueId, HashMap<bool, usize>>,
    ) -> Self {
        let player_name = match attributes.get("Engine.PlayerReplicationInfo:PlayerName") {
            Some(Attribute::String(name)) => name.to_string(),
            _ => {
                error!("Could not find name for player {:?}", attributes);
                "".to_string()
            }
        };

        debug!("Processing player: {}", player_name);

        let mut car_id = None;
        if let Some(Attribute::Loadout(loadout)) = attributes.get("TAGame.PRI_TA:ClientLoadout") {
            car_id = Some(loadout.body as i32);
            debug!(
                "Player '{}' using car_id '{}' from Loadout.",
                player_name,
                car_id.unwrap_or(-1)
            );
        } else if let Some(Attribute::TeamLoadout(loadout)) = attributes.get("TAGame.PRI_TA:ClientLoadouts") {
            car_id = Some(loadout.blue.body as i32);
            debug!(
                "Player '{}' using car_id '{}' from Loadout.",
                player_name,
                car_id.unwrap_or(-1)
            );
        } else {
            warn!("Could not determine car_id for player '{}'. Loadout not attribute found.", player_name);
        }

        let is_orange = match attributes.get("Engine.PlayerReplicationInfo:Team") {
            Some(Attribute::ActiveActor(team_active_actor)) => {
                let team_actor_id = team_active_actor.actor;
                if let Some(team_data) = teams_actor.get(&team_actor_id) {
                    Some(team_data.is_orange)
                } else {
                    try_get_player_team(player_name.as_str(), wrapped_unique_id, players_teams)
                }
            }
            _ => try_get_player_team(player_name.as_str(), wrapped_unique_id, players_teams),
        };

        if is_orange.is_none() {
            warn!("Could not determine team for player '{}'.", player_name);
        }

        Self {
            unique_id: wrapped_unique_id.clone(),
            name: player_name,
            online_id: match attributes.get("Engine.PlayerReplicationInfo:UniqueId") {
                Some(Attribute::UniqueId(unique_id)) => Some(match &unique_id.remote_id {
                    RemoteId::PlayStation(id) => id.online_id.to_string(),
                    RemoteId::Steam(id) => id.to_string(),
                    RemoteId::Switch(id) => id.online_id.to_string(),
                    RemoteId::Epic(id) => id.to_string(),
                    RemoteId::Xbox(id) => id.to_string(),
                    RemoteId::PsyNet(id) => id.online_id.to_string(),
                    RemoteId::SplitScreen(id) => id.to_string(),
                    RemoteId::QQ(id) => id.to_string(),
                }),
                _ => None,
            },
            online_id_kind: match attributes.get("Engine.PlayerReplicationInfo:UniqueId") {
                Some(Attribute::UniqueId(unique_id)) => Some(match &unique_id.remote_id {
                    RemoteId::PlayStation(_) => "PlayStation".to_string(),
                    RemoteId::Steam(_) => "Steam".to_string(),
                    RemoteId::Switch(_) => "Switch".to_string(),
                    RemoteId::Epic(_) => "Epic".to_string(),
                    RemoteId::Xbox(_) => "Xbox".to_string(),
                    RemoteId::PsyNet(_) => "PsyNet".to_string(),
                    RemoteId::SplitScreen(_) => "SplitScreen".to_string(),
                    RemoteId::QQ(_) => "QQ".to_string(),
                }),
                _ => None,
            },
            is_orange,
            car_id,
            match_score: match attributes.get("TAGame.PRI_TA:MatchScore") {
                Some(Attribute::Int(match_score)) => *match_score,
                _ => 0,
            },
            match_goals: match attributes.get("TAGame.PRI_TA:MatchGoals") {
                Some(Attribute::Int(match_goals)) => *match_goals,
                _ => 0,
            },
            match_assists: match attributes.get("TAGame.PRI_TA:MatchAssists") {
                Some(Attribute::Int(match_assists)) => *match_assists,
                _ => 0,
            },
            match_saves: match attributes.get("TAGame.PRI_TA:MatchSaves") {
                Some(Attribute::Int(match_saves)) => *match_saves,
                _ => 0,
            },
            match_shots: match attributes.get("TAGame.PRI_TA:MatchShots") {
                Some(Attribute::Int(match_shots)) => *match_shots,
                _ => 0,
            },
        }
    }
}

fn serialize_wrapped_unique_id<S>(input: &WrappedUniqueId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&input.to_string())
}

/// Tries to get the player's team through FrameParser's running count of whether the team.
/// (This count is only made once every 500 frames)
/// This handles cases where players leave the team at the end (and team_actor_id == ActorId(-1))
/// Only accepts team if at least 3 records were made of the player being in the team, to avoid spectators being recorded as part of the team.
fn try_get_player_team(
    player_name: &str,
    wrapped_unique_id: &WrappedUniqueId,
    players_teams: &HashMap<WrappedUniqueId, HashMap<bool, usize>>,
) -> Option<bool> {
    if let Some(player_teams) = players_teams.get(wrapped_unique_id) {
        debug!(
            "Player '{}' team counts (fallback): {:?}",
            player_name, player_teams
        );
        player_teams
            .iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .and_then(|(k, v)| {
                if v > &3 {
                    Some(*k)
                } else {
                    warn!(
                        "Player '{}' max team count in fallback ({}) is not > 3. Not assigning team.",
                        player_name, v
                    );
                    None
                }
            })
    } else {
        debug!(
            "No team history in players_teams map for player '{}'.",
            player_name
        );
        None
    }
}