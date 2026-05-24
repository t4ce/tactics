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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_state_maps_to_asset_tags() {
        assert_eq!(PlantTree::new(TreeVariant::Two).aseprite_tag(), "Tree 2");
        assert_eq!(
            PlantTree::stump(TreeVariant::Three).aseprite_tag(),
            "Stump 3"
        );
    }

    #[test]
    fn pawn_visual_maps_to_nested_asset_tags() {
        assert_eq!(
            PawnVisual {
                action: PawnAction::Run,
                carry: PawnCarry::Gold,
            }
            .aseprite_tag(),
            "Run Gold"
        );
        assert_eq!(
            PawnVisual {
                action: PawnAction::Interact,
                carry: PawnCarry::Axe,
            }
            .aseprite_tag(),
            "Interact Axe"
        );
    }
}
