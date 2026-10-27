//! Immutable route geometry and traversal policies decoded at the asset boundary.
//! Human progress, steering and ranking intentionally use different views of a
//! waypoint. A redirect still has geometry when the source consumer reads it.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteAxis {
    pub center: u16,
    pub half_extent: u16,
    pub ranking_center: u16,
}
impl RouteAxis {
    fn decode(word: u16) -> Self {
        Self {
            center: word & 0xbfe0,
            half_extent: (word & 31) << 7,
            ranking_center: word & 0xffe0,
        }
    }
    fn encode(self) -> u16 {
        self.ranking_center | (self.half_extent >> 7)
    }
    pub fn contains(self, position: u16) -> bool {
        self.center
            .wrapping_sub(position)
            .wrapping_add(self.half_extent)
            < self.half_extent * 2
    }
    pub fn steer_delta(self, position: u16, bias: u8) -> i8 {
        const UNBIASED_HALF_EXTENT: u16 = 128;
        const BIAS_WORLD_UNITS: i16 = 128;
        const VECTOR_SHIFT: u32 = 6;
        let target = if self.half_extent == UNBIASED_HALF_EXTENT {
            self.center
        } else {
            self.center
                .wrapping_add((i16::from(bias as i8) * BIAS_WORLD_UNITS) as u16)
        };
        // Source keeps only bits6..13 after subtract/shift. The decoded contact
        // center can discard bit14 before bias without changing that byte.
        (target.wrapping_sub(position) >> VECTOR_SHIFT) as u8 as i8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Waypoint {
    pub axes: [RouteAxis; 2],
}
impl Waypoint {
    /// Source asset/contract-fixture boundary, not a gameplay route interpreter.
    pub fn from_encoded(words: [u16; 2]) -> Self {
        Self {
            axes: words.map(RouteAxis::decode),
        }
    }
    pub fn contains(self, position: [u16; 2]) -> bool {
        (0..2).all(|axis| self.axes[axis].contains(position[axis]))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanTraversal {
    Checkpoint,
    Skip,
    Wrap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextTarget {
    Sequential,
    Redirect(u8),
    ConditionalRedirect(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteNode {
    pub x: RouteAxis,
    y: Option<RouteAxis>,
    pub human: HumanTraversal,
    pub next: NextTarget,
    pub choose_bias: bool,
}
impl RouteNode {
    fn decode(x: u16, y: Option<u16>) -> Self {
        let marker = (x >> 8) as u8;
        let human = match marker {
            0xff => HumanTraversal::Wrap,
            0xfd | 0xfe => HumanTraversal::Skip,
            _ if x & 0x4000 != 0 => HumanTraversal::Skip,
            _ => HumanTraversal::Checkpoint,
        };
        let next = match (marker, y) {
            (0xfd, Some(target)) => NextTarget::ConditionalRedirect(target as u8),
            (0xfe, Some(target)) => NextTarget::Redirect(target as u8),
            _ => NextTarget::Sequential,
        };
        let choose_bias = y.is_some_and(|y| ((x | y) & 31) >= 4);
        Self {
            x: RouteAxis::decode(x),
            y: y.map(RouteAxis::decode),
            human,
            next,
            choose_bias,
        }
    }
    pub fn waypoint(&self) -> Result<Waypoint> {
        Ok(Waypoint {
            axes: [self.x, self.y.context("missing route Y")?],
        })
    }
    pub fn ranking_distance(&self, world: [u16; 2]) -> Result<u16> {
        const X_RANK_HALF_EXTENT: u16 = 128;
        let (axis, target) = if self.x.half_extent == X_RANK_HALF_EXTENT {
            (0, self.x)
        } else {
            (1, self.y.context("missing ranking Y")?)
        };
        Ok((target.ranking_center.wrapping_sub(world[axis]) as i16).unsigned_abs())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridOrigin(pub [u16; 2]);
impl From<[u16; 2]> for GridOrigin {
    fn from(words: [u16; 2]) -> Self {
        let waypoint = Waypoint::from_encoded(words);
        Self([waypoint.axes[0].center, waypoint.axes[1].ranking_center])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RouteEncoding", into = "RouteEncoding")]
pub struct Route {
    nodes: Vec<RouteNode>,
    pub checkpoint_count: u8,
    copied_tail: Option<u8>,
}
impl Route {
    pub fn from_words(words: Vec<u16>, checkpoint_count: u8) -> Self {
        let copied_tail = words.get(0xf7 / 2).map(|word| word.to_le_bytes()[1]);
        Self {
            nodes: words
                .chunks(2)
                .map(|pair| RouteNode::decode(pair[0], pair.get(1).copied()))
                .collect(),
            checkpoint_count,
            copied_tail,
        }
    }
    pub fn node(&self, index: u8) -> Result<&RouteNode> {
        self.nodes
            .get(usize::from(index))
            .context("route checkpoint beyond content")
    }
    /// These source consumers have a64-node lookup window, distinct from their
    /// retained byte checkpoint value. Full-index AI/ranking readers use node().
    pub fn short_node(&self, index: u8) -> Result<&RouteNode> {
        const SOURCE_LOOKUP_NODES: u8 = 64;
        self.node(index % SOURCE_LOOKUP_NODES)
    }
    pub fn grid_origin(&self) -> Result<GridOrigin> {
        let waypoint = self.node(0)?.waypoint()?;
        Ok(GridOrigin([
            waypoint.axes[0].center,
            waypoint.axes[1].ranking_center,
        ]))
    }
    pub fn copied_tail(&self) -> Option<u8> {
        self.copied_tail
    }
    pub fn encoded_words(&self) -> Vec<u16> {
        self.nodes
            .iter()
            .flat_map(|node| std::iter::once(node.x.encode()).chain(node.y.map(RouteAxis::encode)))
            .collect()
    }
    pub fn word_len(&self) -> usize {
        self.nodes.len() * 2 - usize::from(self.nodes.last().is_some_and(|node| node.y.is_none()))
    }
}
// Existing TrackContent/TrackFile identities and saved wire values stay exact.
#[derive(Serialize, Deserialize)]
struct RouteEncoding {
    words: Vec<u16>,
    checkpoint_count: u8,
}
impl From<RouteEncoding> for Route {
    fn from(encoded: RouteEncoding) -> Self {
        Self::from_words(encoded.words, encoded.checkpoint_count)
    }
}
impl From<Route> for RouteEncoding {
    fn from(route: Route) -> Self {
        Self {
            words: route.encoded_words(),
            checkpoint_count: route.checkpoint_count,
        }
    }
}
