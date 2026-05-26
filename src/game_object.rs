#![allow(dead_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameObjectKind {
    Rock(RockKind),
    MeatResource,
    WoodResource,
    Bush,
    PlantTree,
    GoldResource,
    GoldStone,
    Pawn,
    Archer,
    Warrior,
    Lancer,
    Monk,
    Sheep,
    RubberDuck,
    ParticleFx,
    WaterFoam,
    WaterRock(WaterRockKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RockKind {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterRockKind {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlantTree {
    pub variant: TreeVariant,
    pub state: TreeState,
}

impl PlantTree {
    pub fn new(variant: TreeVariant) -> Self {
        Self {
            variant,
            state: TreeState::Alive,
        }
    }

    pub fn stump(variant: TreeVariant) -> Self {
        Self {
            variant,
            state: TreeState::Stump,
        }
    }

    pub fn aseprite_tag(self) -> &'static str {
        match (self.variant, self.state) {
            (TreeVariant::One, TreeState::Alive) => "Tree 1",
            (TreeVariant::One, TreeState::Stump) => "Stump 1",
            (TreeVariant::Two, TreeState::Alive) => "Tree 2",
            (TreeVariant::Two, TreeState::Stump) => "Stump 2",
            (TreeVariant::Three, TreeState::Alive) => "Tree 3",
            (TreeVariant::Three, TreeState::Stump) => "Stump 3",
            (TreeVariant::Four, TreeState::Alive) => "Tree 4",
            (TreeVariant::Four, TreeState::Stump) => "Stump 4",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeVariant {
    One,
    Two,
    Three,
    Four,
}

impl TreeVariant {
    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            _ => Self::Four,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeState {
    Alive,
    Stump,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Bush {
    pub variant: BushVariant,
}

impl Bush {
    pub fn aseprite_tag(self) -> &'static str {
        match self.variant {
            BushVariant::One => "Bush 1",
            BushVariant::Two => "Bush 2",
            BushVariant::Three => "Bush 3",
            BushVariant::Four => "Bush 4",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BushVariant {
    One,
    Two,
    Three,
    Four,
}

impl BushVariant {
    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            _ => Self::Four,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PawnVisual {
    pub action: PawnAction,
    pub carry: PawnCarry,
}

impl PawnVisual {
    pub fn aseprite_tag(self) -> &'static str {
        match (self.action, self.carry) {
            (PawnAction::Idle, PawnCarry::None) => "Idle",
            (PawnAction::Idle, PawnCarry::Wood) => "Idle Wood",
            (PawnAction::Idle, PawnCarry::Meat) => "Idle Meat",
            (PawnAction::Idle, PawnCarry::Gold) => "Idle Gold",
            (PawnAction::Idle, PawnCarry::Hammer) => "Idle Hammer",
            (PawnAction::Idle, PawnCarry::Axe) => "Idle Axe",
            (PawnAction::Idle, PawnCarry::Knife) => "Idle Knife",
            (PawnAction::Idle, PawnCarry::Pickaxe) => "Idle Pickaxe",
            (PawnAction::Run, PawnCarry::None) => "Run",
            (PawnAction::Run, PawnCarry::Wood) => "Run Wood",
            (PawnAction::Run, PawnCarry::Meat) => "Run Meat",
            (PawnAction::Run, PawnCarry::Gold) => "Run Gold",
            (PawnAction::Run, PawnCarry::Hammer) => "Run Hammer",
            (PawnAction::Run, PawnCarry::Axe) => "Run Axe",
            (PawnAction::Run, PawnCarry::Knife) => "Run Knife",
            (PawnAction::Run, PawnCarry::Pickaxe) => "Run Pickaxe",
            (PawnAction::Interact, PawnCarry::Hammer) => "Interact Hammer",
            (PawnAction::Interact, PawnCarry::Axe) => "Interact Axe",
            (PawnAction::Interact, PawnCarry::Knife) => "Interact Knife",
            (PawnAction::Interact, PawnCarry::Pickaxe) => "Interact Pickaxe",
            (PawnAction::Interact, _) => "Interact",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PawnAction {
    Idle,
    Run,
    Interact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PawnCarry {
    None,
    Wood,
    Meat,
    Gold,
    Hammer,
    Axe,
    Knife,
    Pickaxe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitAction {
    Idle,
    Run,
    Attack,
    Attack1,
    Attack2,
    Shoot,
    Guard,
    Defense,
    Heal,
}

impl UnitAction {
    pub fn aseprite_tag(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Run => "Run",
            Self::Attack => "Attack",
            Self::Attack1 => "Attack 1",
            Self::Attack2 => "Attack 2",
            Self::Shoot => "Shoot",
            Self::Guard => "Guard",
            Self::Defense => "Defense",
            Self::Heal => "Heal",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AsepriteAnimationRef {
    pub tag_index: usize,
    pub tag_name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LancerAnimation {
    Idle,
    Run,
    Up,
    UpDefense,
    UpAttack,
    UpRight,
    UpRightDefense,
    UpRightAttack,
    Right,
    RightDefense,
    RightAttack,
    DownRight,
    DownRightDefense,
    DownRightAttack,
    Down,
    DownDefense,
    DownAttack,
}

impl LancerAnimation {
    pub const ALL: [Self; 17] = [
        Self::Idle,
        Self::Run,
        Self::Up,
        Self::UpDefense,
        Self::UpAttack,
        Self::UpRight,
        Self::UpRightDefense,
        Self::UpRightAttack,
        Self::Right,
        Self::RightDefense,
        Self::RightAttack,
        Self::DownRight,
        Self::DownRightDefense,
        Self::DownRightAttack,
        Self::Down,
        Self::DownDefense,
        Self::DownAttack,
    ];

    pub fn aseprite_ref(self) -> AsepriteAnimationRef {
        match self {
            Self::Idle => AsepriteAnimationRef {
                tag_index: 0,
                tag_name: "Idle",
            },
            Self::Run => AsepriteAnimationRef {
                tag_index: 1,
                tag_name: "Run",
            },
            Self::Up => AsepriteAnimationRef {
                tag_index: 2,
                tag_name: "Up",
            },
            Self::UpDefense => AsepriteAnimationRef {
                tag_index: 3,
                tag_name: "Defense",
            },
            Self::UpAttack => AsepriteAnimationRef {
                tag_index: 4,
                tag_name: "Attack",
            },
            Self::UpRight => AsepriteAnimationRef {
                tag_index: 5,
                tag_name: "Up Right",
            },
            Self::UpRightDefense => AsepriteAnimationRef {
                tag_index: 6,
                tag_name: "Defense",
            },
            Self::UpRightAttack => AsepriteAnimationRef {
                tag_index: 7,
                tag_name: "Attack",
            },
            Self::Right => AsepriteAnimationRef {
                tag_index: 8,
                tag_name: "Right",
            },
            Self::RightDefense => AsepriteAnimationRef {
                tag_index: 9,
                tag_name: "Defense",
            },
            Self::RightAttack => AsepriteAnimationRef {
                tag_index: 10,
                tag_name: "Attack",
            },
            Self::DownRight => AsepriteAnimationRef {
                tag_index: 11,
                tag_name: "Down Right",
            },
            Self::DownRightDefense => AsepriteAnimationRef {
                tag_index: 12,
                tag_name: "Defense",
            },
            Self::DownRightAttack => AsepriteAnimationRef {
                tag_index: 13,
                tag_name: "Attack",
            },
            Self::Down => AsepriteAnimationRef {
                tag_index: 14,
                tag_name: "Down",
            },
            Self::DownDefense => AsepriteAnimationRef {
                tag_index: 15,
                tag_name: "Defense",
            },
            Self::DownAttack => AsepriteAnimationRef {
                tag_index: 16,
                tag_name: "Attack",
            },
        }
    }
}
