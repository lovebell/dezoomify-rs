use crate::dezoomer::{
    Dezoomer, DezoomerError, DezoomerInput, single_level, TileFetchResult,
    TileProvider, TileReference, ZoomLevels,
};
use crate::Vec2d;

enum Stage {
    Init,
    FirstLine { current_x: u32 },
    NextLines { max_x: u32, current_y: u32 },
}

struct ZoomLevel {
    url_template: String,
    stage: Stage,
    tile_size: Option<Vec2d>,
}

impl ZoomLevel {
    fn tile_url_at(&self, x: u32, y: u32) -> String {
        self.url_template
            .replace("{{X}}", &x.to_string())
            .replace("{{Y}}", &y.to_string())
    }
    fn tile_ref_at(&self, x: u32, y: u32) -> TileReference {
        let tile_size = self.tile_size.unwrap_or(Vec2d { x: 0, y: 0 });
        let position = Vec2d { x, y } * tile_size;
        TileReference {
            url: self.tile_url_at(x, y),
            position,
        }
    }
}

impl TileProvider for ZoomLevel {
    fn next_tiles(&mut self, previous: Option<TileFetchResult>) -> Vec<TileReference> {
        match (previous, &self.stage) {
            // First request
            (None, _) => vec![self.tile_ref_at(0, 0)],

            // First request failed
            (Some(ref res), Stage::Init) if !res.is_success() => vec![],

            // Switch from Init to FirstLine
            (Some(TileFetchResult { tile_size, .. }), Stage::Init) => {
                self.stage = Stage::FirstLine { current_x: 1 };
                self.tile_size = tile_size;
                vec![self.tile_ref_at(1, 0)]
            }

            // Advance in the first line
            (Some(ref res), &Stage::FirstLine { current_x }) if res.is_success() => {
                let current_x = current_x + 1;
                self.stage = Stage::FirstLine { current_x };
                vec![self.tile_ref_at(current_x, 0)]
            }

            // End of first line
            (Some(_), &Stage::FirstLine { current_x }) => {
                let max_x = current_x - 1;
                self.stage = Stage::NextLines {
                    max_x,
                    current_y: 1,
                };
                (0..=max_x).map(|x| self.tile_ref_at(x, 1)).collect()
            }

            // Advance to next line
            (Some(ref res), &Stage::NextLines { current_y, max_x }) if res.is_success() => {
                let current_y = current_y + 1;
                self.stage = Stage::NextLines { max_x, current_y };
                (0..=max_x)
                    .map(|x| self.tile_ref_at(x, current_y))
                    .collect()
            }

            // End of image
            (Some(_), Stage::NextLines { .. }) => vec![],
        }
    }

    fn name(&self) -> String {
        format!("Generic image with template {}", self.url_template)
    }
}

impl std::fmt::Debug for ZoomLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Generic level")
    }
}

#[derive(Default)]
pub struct GenericDezoomer;

impl Dezoomer for GenericDezoomer {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn zoom_levels(&mut self, data: &DezoomerInput) -> Result<ZoomLevels, DezoomerError> {
        self.assert(data.uri.contains("{{X}}"))?;
        let dezoomer = ZoomLevel {
            url_template: data.uri.clone(),
            stage: Stage::Init,
            tile_size: None,
        };
        single_level(dezoomer)
    }
}

#[test]
fn test_generic_dezoomer() {
    let uri = "{{X}},{{Y}}".to_string();
    let mut lvl = GenericDezoomer {}
        .zoom_levels(&DezoomerInput {
            uri,
            contents: None,
        })
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let existing_tiles = vec!["0,0", "1,0", "2,0", "0,1", "1,1", "2,1"];

    let mut all_tiles = vec![];

    crate::dezoomer::apply_to_tiles(&mut lvl, |tiles| {
        let count = tiles.len() as u64;

        let successes: Vec<_> = tiles.into_iter()
            .filter(|t| existing_tiles.contains(&t.url.as_str()))
            .collect();
        let res = TileFetchResult {
            count,
            successes: successes.len() as u64,
            tile_size: Some(Vec2d { x: 4, y: 5 }),
        };
        all_tiles.extend(successes);
        res
    });

    assert_eq!(all_tiles, vec![
        TileReference { url: "0,0".into(), position: Vec2d { x: 0, y: 0 } },
        TileReference { url: "1,0".into(), position: Vec2d { x: 4, y: 0 } },
        TileReference { url: "2,0".into(), position: Vec2d { x: 8, y: 0 } },
        TileReference { url: "0,1".into(), position: Vec2d { x: 0, y: 5 } },
        TileReference { url: "1,1".into(), position: Vec2d { x: 4, y: 5 } },
        TileReference { url: "2,1".into(), position: Vec2d { x: 8, y: 5 } },
    ])
}
