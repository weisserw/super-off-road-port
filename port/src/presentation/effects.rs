//! Native effect clips and owned playback. Source entry numbers are confined to
//! the asset/save adapter; gameplay starts clips and asks lifecycle questions.
use anyhow::{Result, ensure};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Idle,
    Nitro(u8),
    Surface(u8),
    SurfaceRepeat(u8),
    Contact(u8),
    InvalidLegacy(u8),
}
/// Includes terminal frames: they remain observable until the next service.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct AnimationPlayer(Phase);
impl From<u8> for AnimationPlayer {
    fn from(entry: u8) -> Self {
        Self(match entry {
            0 => Phase::Idle,
            1..=9 => Phase::Nitro(entry - 1),
            10..=18 => Phase::Surface(entry - 10),
            19..=23 => Phase::SurfaceRepeat(entry - 19),
            24..=30 => Phase::Contact(entry - 24),
            _ => Phase::InvalidLegacy(entry),
        })
    }
}
impl From<AnimationPlayer> for u8 {
    fn from(player: AnimationPlayer) -> Self {
        player.encoded()
    }
}
impl AnimationPlayer {
    pub fn encoded(self) -> u8 {
        match self.0 {
            Phase::Idle => 0,
            Phase::Nitro(frame) => 1 + frame,
            Phase::Surface(frame) => 10 + frame,
            Phase::SurfaceRepeat(frame) => 19 + frame,
            Phase::Contact(frame) => 24 + frame,
            Phase::InvalidLegacy(entry) => entry,
        }
    }
    pub fn valid(self) -> bool {
        !matches!(self.0, Phase::InvalidLegacy(_))
    }
    pub fn is_idle(self) -> bool {
        self.0 == Phase::Idle
    }
    pub fn can_resume_surface(self) -> bool {
        matches!(self.0, Phase::Surface(8) | Phase::SurfaceRepeat(4))
    }
    pub fn start_nitro(&mut self) {
        self.0 = Phase::Nitro(0);
    }
    pub fn start_contact(&mut self) {
        self.0 = Phase::Contact(0);
    }
    pub fn start_surface(&mut self) {
        self.0 = Phase::Surface(0);
    }
    pub fn resume_surface(&mut self) {
        self.0 = Phase::SurfaceRepeat(0);
    }
    pub fn clear(&mut self) {
        self.0 = Phase::Idle;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Hold,
    Finish,
    Draw {
        tile_attributes: [u8; 2],
        next: AnimationPlayer,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationClips {
    idle: Action,
    nitro: [Action; 9],
    surface: [Action; 9],
    surface_repeat: [Action; 5],
    contact: [Action; 7],
}
impl AnimationClips {
    /// Decode all entries, including overridden terminal/hold commands. A draw
    /// in the final source cell advances to a deferred invalid legacy state,
    /// preserving the old next-service error rather than rejecting the asset.
    pub fn decode(tiles: &[u8; 31], attributes: &[u8; 31]) -> Self {
        let decode = |entry: usize| match tiles[entry] {
            0xfe => Action::Hold,
            0xff => Action::Finish,
            tile => Action::Draw {
                tile_attributes: [tile, attributes[entry]],
                next: AnimationPlayer::from(entry as u8 + 1),
            },
        };
        Self {
            idle: decode(0),
            nitro: std::array::from_fn(|frame| decode(frame + 1)),
            surface: std::array::from_fn(|frame| decode(frame + 10)),
            surface_repeat: std::array::from_fn(|frame| decode(frame + 19)),
            contact: std::array::from_fn(|frame| decode(frame + 24)),
        }
    }
    /// Caller validates every owner before starting the complete staging pass.
    pub fn stage(&self, player: &mut AnimationPlayer, frame: u8, oam: &mut [u8]) -> Result<()> {
        ensure!(player.valid(), "invalid animation cursor");
        let action = match player.0 {
            Phase::Idle => self.idle,
            Phase::Nitro(frame) => self.nitro[usize::from(frame)],
            Phase::Surface(frame) => self.surface[usize::from(frame)],
            Phase::SurfaceRepeat(frame) => self.surface_repeat[usize::from(frame)],
            Phase::Contact(frame) => self.contact[usize::from(frame)],
            Phase::InvalidLegacy(_) => unreachable!(),
        };
        match action {
            Action::Hold => {}
            Action::Finish => {
                oam[1] = super::HIDDEN_Y;
                player.clear();
            }
            Action::Draw {
                tile_attributes,
                next,
            } => {
                oam[2..4].copy_from_slice(&tile_attributes);
                if frame & 3 == 0 {
                    *player = next;
                }
            }
        }
        Ok(())
    }
}
