#![allow(dead_code)]

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AtlasTile {
    pub col: u32,
    pub row: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TerrainTileId(usize);

impl TerrainTileId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub const CARDINALS: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub fn label(self) -> &'static str {
        match self {
            Self::North => "N",
            Self::East => "E",
            Self::South => "S",
            Self::West => "W",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Socket {
    Empty,
    BoxA,
    BoxB,
    OptionalDrop,
    Row,
    OptionalRow,
    StripA,
    OptionalStripA,
    StripB,
    OptionalStripB,
}

impl Socket {
    fn label(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::BoxA => "box_a",
            Self::BoxB => "box_b",
            Self::OptionalDrop => "optional_drop",
            Self::Row => "row",
            Self::OptionalRow => "optional_row",
            Self::StripA => "strip_a",
            Self::OptionalStripA => "optional_strip_a",
            Self::StripB => "strip_b",
            Self::OptionalStripB => "optional_strip_b",
        }
    }

    fn accepts_empty(self) -> bool {
        matches!(
            self,
            Self::Empty
                | Self::OptionalDrop
                | Self::OptionalRow
                | Self::OptionalStripA
                | Self::OptionalStripB
        )
    }

    fn touches(self, other: Self) -> bool {
        match (self, other) {
            (Self::Row | Self::OptionalRow, Self::Row | Self::OptionalRow) => true,
            (Self::StripA | Self::OptionalStripA, Self::StripA | Self::OptionalStripA) => true,
            (Self::StripB | Self::OptionalStripB, Self::StripB | Self::OptionalStripB) => true,
            _ => self != Self::Empty && self == other,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Sockets {
    north: Socket,
    east: Socket,
    south: Socket,
    west: Socket,
}

impl Sockets {
    fn get(self, direction: Direction) -> Socket {
        match direction {
            Direction::North => self.north,
            Direction::East => self.east,
            Direction::South => self.south,
            Direction::West => self.west,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TerrainRule {
    pub name: String,
    pub atlas_tile: AtlasTile,
    sockets: Sockets,
}

#[derive(Clone, Debug)]
pub struct TerrainRules {
    rules: Vec<TerrainRule>,
}

fn push_box_family(
    rules: &mut Vec<TerrainRule>,
    name: &str,
    start_col: u32,
    start_row: u32,
    socket: Socket,
) {
    for row in 0..3 {
        for col in 0..3 {
            rules.push(TerrainRule {
                name: format!("{name}_{col}_{row}"),
                atlas_tile: AtlasTile {
                    col: start_col + col,
                    row: start_row + row,
                },
                sockets: Sockets {
                    north: if row == 0 { Socket::Empty } else { socket },
                    east: if col == 2 { Socket::Empty } else { socket },
                    south: if row == 2 {
                        Socket::OptionalDrop
                    } else {
                        socket
                    },
                    west: if col == 0 { Socket::Empty } else { socket },
                },
            });
        }
    }
}

fn push_row_family(rules: &mut Vec<TerrainRule>, name: &str, row: u32) {
    for col in 5..8 {
        rules.push(TerrainRule {
            name: format!("{name}_{}", col - 5),
            atlas_tile: AtlasTile { col, row },
            sockets: Sockets {
                north: Socket::OptionalDrop,
                east: match col {
                    5 => Socket::Row,
                    6 => Socket::OptionalRow,
                    _ => Socket::Empty,
                },
                south: Socket::Empty,
                west: match col {
                    5 => Socket::Empty,
                    6 => Socket::OptionalRow,
                    _ => Socket::Row,
                },
            },
        });
    }
}

fn push_vertical_strip_family(rules: &mut Vec<TerrainRule>, name: &str, col: u32, socket: Socket) {
    for row in 0..3 {
        rules.push(TerrainRule {
            name: format!("{name}_v_{row}"),
            atlas_tile: AtlasTile { col, row },
            sockets: Sockets {
                north: match row {
                    0 => Socket::Empty,
                    _ => socket,
                },
                east: Socket::Empty,
                south: match row {
                    2 => Socket::Empty,
                    _ => socket,
                },
                west: Socket::Empty,
            },
        });
    }
}

fn push_horizontal_strip_family(
    rules: &mut Vec<TerrainRule>,
    name: &str,
    row: u32,
    start_col: u32,
    socket: Socket,
) {
    for offset in 0..3 {
        let col = start_col + offset;
        rules.push(TerrainRule {
            name: format!("{name}_h_{offset}"),
            atlas_tile: AtlasTile { col, row },
            sockets: Sockets {
                north: Socket::Empty,
                east: match offset {
                    2 => Socket::Empty,
                    _ => socket,
                },
                south: Socket::Empty,
                west: match offset {
                    0 => Socket::Empty,
                    _ => socket,
                },
            },
        });
    }
}

fn push_strip_single(
    rules: &mut Vec<TerrainRule>,
    name: &str,
    atlas_tile: AtlasTile,
    optional_socket: Socket,
) {
    rules.push(TerrainRule {
        name: format!("{name}_single"),
        atlas_tile,
        sockets: Sockets {
            north: optional_socket,
            east: optional_socket,
            south: optional_socket,
            west: optional_socket,
        },
    });
}

impl TerrainRules {
    pub fn color2() -> Self {
        let mut rules = Vec::new();

        push_box_family(&mut rules, "box_a", 0, 0, Socket::BoxA);
        push_box_family(&mut rules, "box_b", 5, 0, Socket::BoxB);
        push_vertical_strip_family(&mut rules, "strip_a", 3, Socket::StripA);
        push_horizontal_strip_family(&mut rules, "strip_a", 3, 0, Socket::StripA);
        push_strip_single(
            &mut rules,
            "strip_a",
            AtlasTile { col: 3, row: 3 },
            Socket::OptionalStripA,
        );
        push_vertical_strip_family(&mut rules, "strip_b", 8, Socket::StripB);
        push_horizontal_strip_family(&mut rules, "strip_b", 3, 5, Socket::StripB);
        push_strip_single(
            &mut rules,
            "strip_b",
            AtlasTile { col: 8, row: 3 },
            Socket::OptionalStripB,
        );
        push_row_family(&mut rules, "row_a", 4);
        push_row_family(&mut rules, "row_b", 5);

        Self { rules }
    }

    pub fn rule(&self, id: TerrainTileId) -> &TerrainRule {
        &self.rules[id.0]
    }

    pub fn iter(&self) -> impl Iterator<Item = (TerrainTileId, &TerrainRule)> {
        self.rules
            .iter()
            .enumerate()
            .map(|(index, rule)| (TerrainTileId(index), rule))
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn id_for_atlas_tile(&self, atlas_tile: AtlasTile) -> Option<TerrainTileId> {
        self.rules
            .iter()
            .position(|rule| rule.atlas_tile == atlas_tile)
            .map(TerrainTileId)
    }

    pub fn can_touch(
        &self,
        a: Option<TerrainTileId>,
        direction_from_a: Direction,
        b: Option<TerrainTileId>,
    ) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), None) => self.rule(a).sockets.get(direction_from_a).accepts_empty(),
            (None, Some(b)) => self
                .rule(b)
                .sockets
                .get(direction_from_a.opposite())
                .accepts_empty(),
            (Some(a), Some(b)) => {
                let a_socket = self.rule(a).sockets.get(direction_from_a);
                let b_socket = self.rule(b).sockets.get(direction_from_a.opposite());
                a_socket.touches(b_socket)
            }
        }
    }

    pub fn adjacency_matrix(&self, direction_from_a: Direction) -> Vec<Vec<bool>> {
        let mut matrix = vec![vec![false; self.rules.len()]; self.rules.len()];
        for (a, row) in matrix.iter_mut().enumerate() {
            for (b, allowed) in row.iter_mut().enumerate() {
                *allowed = self.can_touch(
                    Some(TerrainTileId(a)),
                    direction_from_a,
                    Some(TerrainTileId(b)),
                );
            }
        }
        matrix
    }

    pub fn debug_report(&self) -> String {
        let mut out = String::new();
        out.push_str("Terrain rules for Tilemap_color2.png\n");
        out.push_str(
            "A neighbor can touch when facing sockets match. Empty means exposed edge.\n\n",
        );

        for (id, rule) in self.rules.iter().enumerate() {
            out.push_str(&format!(
                "#{id:02} {:12} atlas=({},{})  N={:<10} E={:<10} S={:<10} W={:<10}\n",
                rule.name,
                rule.atlas_tile.col,
                rule.atlas_tile.row,
                rule.sockets.north.label(),
                rule.sockets.east.label(),
                rule.sockets.south.label(),
                rule.sockets.west.label(),
            ));
        }

        out.push('\n');
        for direction in Direction::CARDINALS {
            out.push_str(&format!(
                "Allowed neighbors to the {} side:\n",
                direction.label()
            ));
            for id in 0..self.rules.len() {
                let allowed = (0..self.rules.len())
                    .filter(|&neighbor| {
                        self.can_touch(
                            Some(TerrainTileId(id)),
                            direction,
                            Some(TerrainTileId(neighbor)),
                        )
                    })
                    .map(|neighbor| format!("#{neighbor:02}"))
                    .collect::<Vec<_>>();
                out.push_str(&format!(
                    "  #{id:02} {:12} -> {}\n",
                    self.rules[id].name,
                    if allowed.is_empty() {
                        "none".to_string()
                    } else {
                        allowed.join(" ")
                    }
                ));
            }
            out.push('\n');
        }

        out
    }
}
