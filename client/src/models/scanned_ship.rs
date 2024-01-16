/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.   
 *
 * The version of the OpenAPI document: 2.0.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */

/// ScannedShip : The ship that was scanned. Details include information about the ship that could be detected by the scanner.



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScannedShip {
    /// The globally unique identifier of the ship.
    #[serde(rename = "symbol")]
    pub symbol: String,
    #[serde(rename = "registration")]
    pub registration: Box<crate::models::ShipRegistration>,
    #[serde(rename = "nav")]
    pub nav: Box<crate::models::ShipNav>,
    #[serde(rename = "frame", skip_serializing_if = "Option::is_none")]
    pub frame: Option<Box<crate::models::ScannedShipFrame>>,
    #[serde(rename = "reactor", skip_serializing_if = "Option::is_none")]
    pub reactor: Option<Box<crate::models::ScannedShipReactor>>,
    #[serde(rename = "engine")]
    pub engine: Box<crate::models::ScannedShipEngine>,
    /// List of mounts installed in the ship.
    #[serde(rename = "mounts", skip_serializing_if = "Option::is_none")]
    pub mounts: Option<Vec<crate::models::ScannedShipMountsInner>>,
}

impl ScannedShip {
    /// The ship that was scanned. Details include information about the ship that could be detected by the scanner.
    pub fn new(symbol: String, registration: crate::models::ShipRegistration, nav: crate::models::ShipNav, engine: crate::models::ScannedShipEngine) -> ScannedShip {
        ScannedShip {
            symbol,
            registration: Box::new(registration),
            nav: Box::new(nav),
            frame: None,
            reactor: None,
            engine: Box::new(engine),
            mounts: None,
        }
    }
}


