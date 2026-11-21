//! The clear-fluid glass's six face connections: the pure rule that decides them.
//!
//! `clear_fluid_glass` and `clear_fluid_fake_glass` each declare six `boolean` state properties
//! (`up`, `down`, `east`, `west`, `north`, `south`) that select which face models the multipart
//! blockstate draws. Upstream (`BigGlassBlockEntity`) derives them, in `getStateForPlacement`
//! and in `updateShape`, from the block beside each face: `neighbourState.is(this)`, so a face
//! connects exactly when the block there is the *same* block. This module is that rule without
//! the game: [`Face`] names a face's property and its step, and [`Links`] holds the six answers.

/// One face of a block, with the state property that records it and the direction it lies in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    /// The face below the block (`down`).
    Down,
    /// The face above the block (`up`).
    Up,
    /// The face towards negative Z (`north`).
    North,
    /// The face towards positive Z (`south`).
    South,
    /// The face towards negative X (`west`).
    West,
    /// The face towards positive X (`east`).
    East,
}

impl Face {
    /// The six faces, in the order the game's own direction list enumerates them.
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    /// This face's position in [`Face::ALL`], which is also its index in [`Links`].
    const fn index(self) -> usize {
        self as usize
    }

    /// The declared state property that records whether this face connects.
    pub const fn property(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

    /// The step from the block to the neighbour this face connects to.
    pub const fn offset(self) -> [i32; 3] {
        match self {
            Self::Down => [0, -1, 0],
            Self::Up => [0, 1, 0],
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
            Self::East => [1, 0, 0],
        }
    }
}

/// Which of a clear-fluid glass's six faces connect to the same block beside them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Links([bool; 6]);

impl Links {
    /// The six answers of `same_block`, which is asked once per face whether the block beside
    /// that face is the same block as this one.
    pub fn from_connected(mut same_block: impl FnMut(Face) -> bool) -> Self {
        let mut connected = [false; 6];
        for face in Face::ALL {
            connected[face.index()] = same_block(face);
        }
        Self(connected)
    }

    /// Whether `face` connects to the block beside it.
    pub const fn is_connected(self, face: Face) -> bool {
        self.0[face.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One glass beside the block it shares a face with: the two faces of the pair connect and
    /// the other four do not, whichever pair it is.
    #[test]
    fn a_face_links_only_where_the_block_beside_it_is_the_same_block() {
        let beside = [Face::North, Face::South];
        let links = Links::from_connected(|face| beside.contains(&face));
        for face in Face::ALL {
            assert_eq!(
                links.is_connected(face),
                beside.contains(&face),
                "face {face:?}"
            );
        }

        let none = Links::from_connected(|_| false);
        let all = Links::from_connected(|_| true);
        for face in Face::ALL {
            assert!(!none.is_connected(face), "face {face:?} with no neighbour");
            assert!(all.is_connected(face), "face {face:?} with every neighbour");
        }
    }

    /// The property names have to be the ones the carried declarations declare: a mismatch would
    /// leave the boolean false for ever, without a compile error anywhere.
    #[test]
    fn the_six_faces_name_the_properties_both_declarations_declare() {
        for declaration in [
            include_str!("../data/glassential/horizon/block/clear_fluid_glass.json"),
            include_str!("../data/glassential/horizon/block/clear_fluid_fake_glass.json"),
        ] {
            assert_eq!(
                declaration.matches("\"type\": \"boolean\"").count(),
                6,
                "the declaration must declare six booleans"
            );
            for face in Face::ALL {
                let property = format!("\"{}\": {{", face.property());
                assert!(
                    declaration.contains(&property),
                    "the declaration does not declare {property} for {face:?}"
                );
            }
        }
    }

    /// A swapped offset would connect the wrong neighbour: each face steps towards the side it is
    /// named after, and no two faces share a step.
    #[test]
    fn each_face_steps_towards_the_neighbour_it_names() {
        assert_eq!(Face::Down.offset(), [0, -1, 0]);
        assert_eq!(Face::Up.offset(), [0, 1, 0]);
        assert_eq!(Face::North.offset(), [0, 0, -1]);
        assert_eq!(Face::South.offset(), [0, 0, 1]);
        assert_eq!(Face::West.offset(), [-1, 0, 0]);
        assert_eq!(Face::East.offset(), [1, 0, 0]);
        let mut seen: Vec<[i32; 3]> = Face::ALL.iter().map(|face| face.offset()).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 6, "two faces share a direction");
    }
}
