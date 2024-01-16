/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.   
 *
 * The version of the OpenAPI document: 2.0.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */

/// ShipyardShip : 



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipyardShip {
    #[serde(rename = "type")]
    pub r#type: crate::models::ShipType,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "supply")]
    pub supply: crate::models::SupplyLevel,
    #[serde(rename = "activity", skip_serializing_if = "Option::is_none")]
    pub activity: Option<crate::models::ActivityLevel>,
    #[serde(rename = "purchasePrice")]
    pub purchase_price: i32,
    #[serde(rename = "frame")]
    pub frame: Box<crate::models::ShipFrame>,
    #[serde(rename = "reactor")]
    pub reactor: Box<crate::models::ShipReactor>,
    #[serde(rename = "engine")]
    pub engine: Box<crate::models::ShipEngine>,
    #[serde(rename = "modules")]
    pub modules: Vec<crate::models::ShipModule>,
    #[serde(rename = "mounts")]
    pub mounts: Vec<crate::models::ShipMount>,
    #[serde(rename = "crew")]
    pub crew: Box<crate::models::ShipyardShipCrew>,
}

impl ShipyardShip {
    /// 
    pub fn new(r#type: crate::models::ShipType, name: String, description: String, supply: crate::models::SupplyLevel, purchase_price: i32, frame: crate::models::ShipFrame, reactor: crate::models::ShipReactor, engine: crate::models::ShipEngine, modules: Vec<crate::models::ShipModule>, mounts: Vec<crate::models::ShipMount>, crew: crate::models::ShipyardShipCrew) -> ShipyardShip {
        ShipyardShip {
            r#type,
            name,
            description,
            supply,
            activity: None,
            purchase_price,
            frame: Box::new(frame),
            reactor: Box::new(reactor),
            engine: Box::new(engine),
            modules,
            mounts,
            crew: Box::new(crew),
        }
    }
}


