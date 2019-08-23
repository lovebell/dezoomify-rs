use std::collections::HashMap;

use serde::Deserialize;

use crate::default_headers;
use crate::dezoomer::*;
use crate::TileReference;

mod tile_set;
mod variable;

#[derive(Deserialize)]
struct CustomYamlTiles {
    #[serde(flatten)]
    tile_set: tile_set::TileSet,
    #[serde(default = "default_headers")]
    headers: HashMap<String, String>,
}

impl std::fmt::Debug for CustomYamlTiles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Custom tiles")
    }
}

impl TileProvider for CustomYamlTiles {
    fn next_tiles(&mut self, previous: Option<TileFetchResult>) -> Vec<TileReference> {
        if previous.is_some() {
            return vec![];
        }
        self.tile_set
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("Invalid tiles")
    }

    fn http_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
}

#[derive(Default)]
pub struct CustomDezoomer;

impl Dezoomer for CustomDezoomer {
    fn name(&self) -> &'static str {
        "custom"
    }

    fn zoom_levels(&mut self, data: &DezoomerInput) -> Result<ZoomLevels, DezoomerError> {
        self.assert(data.uri.ends_with("tiles.yaml"))?;
        let contents = data.with_contents()?.contents;
        let dezoomer: CustomYamlTiles =
            serde_yaml::from_slice(&contents).map_err(DezoomerError::wrap)?;
        single_level(dezoomer)
    }
}

#[test]
fn test_can_parse_example() {
    use std::fs::File;

    let yaml_path = format!("{}/tiles.yaml", env!("CARGO_MANIFEST_DIR"));
    let file = File::open(yaml_path).unwrap();
    let conf: CustomYamlTiles = serde_yaml::from_reader(file).unwrap();
    assert!(
        conf.http_headers().contains_key("Referer"),
        "There should be a referer in the example"
    );
}

#[test]
fn test_has_default_user_agent() {
    let conf: CustomYamlTiles =
        serde_yaml::from_str("url_template: test.com\nvariables: []").unwrap();
    assert!(
        conf.http_headers().contains_key("User-Agent"),
        "There should be a user agent"
    );
}
