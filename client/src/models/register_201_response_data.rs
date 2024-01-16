/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.   
 *
 * The version of the OpenAPI document: 2.0.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Register201ResponseData {
    #[serde(rename = "agent")]
    pub agent: Box<crate::models::Agent>,
    #[serde(rename = "contract")]
    pub contract: Box<crate::models::Contract>,
    #[serde(rename = "faction")]
    pub faction: Box<crate::models::Faction>,
    #[serde(rename = "ship")]
    pub ship: Box<crate::models::Ship>,
    /// A Bearer token for accessing secured API endpoints.
    #[serde(rename = "token")]
    pub token: String,
}

impl Register201ResponseData {
    pub fn new(agent: crate::models::Agent, contract: crate::models::Contract, faction: crate::models::Faction, ship: crate::models::Ship, token: String) -> Register201ResponseData {
        Register201ResponseData {
            agent: Box::new(agent),
            contract: Box::new(contract),
            faction: Box::new(faction),
            ship: Box::new(ship),
            token,
        }
    }
}


