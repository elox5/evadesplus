use crate::{
    game::{
        area::AreaKey,
        player::PlayerId,
        portal::{PortalTargetPosX, PortalTargetPosY},
    },
    physics::rect::Rect,
};

#[derive(Clone, Debug)]
pub struct TransferRequest {
    pub player: PlayerId,
    pub target: TransferTarget,
    pub target_pos: Option<TransferRequestTargetPos>,
}

#[derive(Clone, Debug)]
pub enum TransferTarget {
    Area(AreaKey),
    MapStart(String),
    Spawn,
}

#[derive(Clone, Debug)]
pub struct TransferRequestTargetPos {
    pub x: TransferRequestTargetPosX,
    pub y: TransferRequestTargetPosY,
}

#[derive(Clone, Debug)]
pub enum TransferRequestTargetPosX {
    FromLeft(f32),
    FromRight(f32),
    Center,
    Resolved(f32),
}

impl TransferRequestTargetPosX {
    pub fn new(data: PortalTargetPosX, player_x: f32) -> Self {
        match data {
            PortalTargetPosX::FromLeft(x) => Self::FromLeft(x),
            PortalTargetPosX::FromRight(x) => Self::FromRight(x),
            PortalTargetPosX::Center => Self::Center,
            PortalTargetPosX::KeepPlayer => Self::Resolved(player_x),
        }
    }

    pub fn resolve(&self, bounds: &Rect) -> f32 {
        match self {
            TransferRequestTargetPosX::FromLeft(x) => *x,
            TransferRequestTargetPosX::FromRight(x) => bounds.right() - x,
            TransferRequestTargetPosX::Center => bounds.center().x,
            TransferRequestTargetPosX::Resolved(x) => *x,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TransferRequestTargetPosY {
    FromBottom(f32),
    FromTop(f32),
    Center,
    Resolved(f32),
}

impl TransferRequestTargetPosY {
    pub fn new(data: PortalTargetPosY, player_y: f32) -> Self {
        match data {
            PortalTargetPosY::FromBottom(y) => Self::FromBottom(y),
            PortalTargetPosY::FromTop(y) => Self::FromTop(y),
            PortalTargetPosY::Center => Self::Center,
            PortalTargetPosY::KeepPlayer => Self::Resolved(player_y),
        }
    }

    pub fn resolve(&self, bounds: &Rect) -> f32 {
        match self {
            TransferRequestTargetPosY::FromBottom(x) => *x,
            TransferRequestTargetPosY::FromTop(x) => bounds.top() - x,
            TransferRequestTargetPosY::Center => bounds.center().y,
            TransferRequestTargetPosY::Resolved(x) => *x,
        }
    }
}
