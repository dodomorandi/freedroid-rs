use crate::{
    array_c_string::ArrayCString,
    defs::{
        self, DisplayBannerFlags, Droid, MenuAction, Status, DROID_ROTATION_TIME, SHOW_WAIT, UPDATE,
    },
    graphics::Graphics,
    structs::Point,
};

use cstr::cstr;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};
use sdl::{convert::u8_to_usize, Rect, Surface};
use sdl_sys::SDL_Color;
use std::{
    convert::Infallible,
    ffi::CStr,
    ops::{Deref, DerefMut, Not},
};

#[derive(Debug)]
struct Map<T>([map::Line<T>; COLORS]);

mod map {
    use std::{
        iter::IntoIterator,
        ops::{Deref, DerefMut, Index, IndexMut},
    };

    use super::{NUM_LAYERS, NUM_LINES};

    macro_rules! impl_traits {
        ($ty:ident, $inner:ident) => {
            impl<T> From<$inner<T>> for $ty<T> {
                fn from(inner: $inner<T>) -> Self {
                    Self(inner)
                }
            }

            impl<T> AsRef<$inner<T>> for $ty<T> {
                fn as_ref(&self) -> &$inner<T> {
                    &self.0
                }
            }

            impl<T> AsMut<$inner<T>> for $ty<T> {
                fn as_mut(&mut self) -> &mut $inner<T> {
                    &mut self.0
                }
            }

            impl<I, T> Index<I> for $ty<T>
            where
                $inner<T>: Index<I>,
            {
                type Output = <$inner<T> as Index<I>>::Output;

                fn index(&self, idx: I) -> &Self::Output {
                    &self.0[idx]
                }
            }

            impl<I, T> IndexMut<I> for $ty<T>
            where
                $inner<T>: IndexMut<I>,
            {
                fn index_mut(&mut self, idx: I) -> &mut Self::Output {
                    &mut self.0[idx]
                }
            }

            impl<T> IntoIterator for $ty<T> {
                type Item = <$inner<T> as IntoIterator>::Item;
                type IntoIter = <$inner<T> as IntoIterator>::IntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    IntoIterator::into_iter(self.0)
                }
            }

            impl<'a, T> IntoIterator for &'a $ty<T> {
                type Item = <&'a $inner<T> as IntoIterator>::Item;
                type IntoIter = <&'a $inner<T> as IntoIterator>::IntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    IntoIterator::into_iter(&self.0)
                }
            }

            impl<'a, T> IntoIterator for &'a mut $ty<T> {
                type Item = <&'a mut $inner<T> as IntoIterator>::Item;
                type IntoIter = <&'a mut $inner<T> as IntoIterator>::IntoIter;

                fn into_iter(self) -> Self::IntoIter {
                    IntoIterator::into_iter(&mut self.0)
                }
            }

            impl<T> $ty<T> {
                pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
                    IntoIterator::into_iter(&self.0)
                }

                pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
                    IntoIterator::into_iter(&mut self.0)
                }

                pub fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
                    IntoIterator::into_iter(self.0)
                }
            }

            impl<T> Deref for $ty<T> {
                type Target = $inner<T>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<T> DerefMut for $ty<T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        };
    }

    type LineInner<T> = [Layer<T>; NUM_LAYERS];

    #[derive(Debug)]
    pub struct Line<T>(LineInner<T>);

    impl<T> Line<T> {
        pub fn proximal_connection(&self) -> &Layer<T> {
            &self.0[2]
        }

        pub fn distal_connection(&self) -> &Layer<T> {
            &self.0[3]
        }

        pub fn layers_mut(&mut self) -> [&mut Layer<T>; NUM_LAYERS] {
            let [a, b, c, d] = &mut self.0;
            [a, b, c, d]
        }
    }

    type LayerInner<T> = [T; NUM_LINES];

    #[derive(Debug)]
    pub struct Layer<T>(LayerInner<T>);

    impl_traits!(Line, LineInner);
    impl_traits!(Layer, LayerInner);
}

impl<T> From<[[[T; NUM_LINES]; NUM_LAYERS]; COLORS]> for Map<T> {
    fn from(map: [[[T; NUM_LINES]; NUM_LAYERS]; COLORS]) -> Self {
        Self(map.map(|line| line.map(map::Layer::from).into()))
    }
}

impl<T> AsRef<<Self as MapExt>::Map> for Map<T> {
    fn as_ref(&self) -> &<Self as MapExt>::Map {
        &self.0
    }
}

impl<T> AsMut<<Self as MapExt>::Map> for Map<T> {
    fn as_mut(&mut self) -> &mut <Self as MapExt>::Map {
        &mut self.0
    }
}

impl<T> Deref for Map<T> {
    type Target = <Self as MapExt>::Map;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Map<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

trait MapExt {
    type Item;
    type Layer;
    type Line;
    type Map;
    type RawMap;
}

impl<T> MapExt for Map<T> {
    type Item = T;
    type Layer = [T; NUM_LINES];
    type Line = [Self::Layer; NUM_LAYERS];
    type Map = [map::Line<T>; COLORS];
    type RawMap = [Self::Line; COLORS];
}

type Playground = Map<Block>;
type ActivationMap = Map<Condition>;
type CapsulesCountdown = Map<Option<u8>>;

#[derive(Debug)]
pub struct Takeover<'sdl> {
    capsule_cur_row: [i32; COLORS],
    num_capsules: [i32; COLORS],
    playground: Playground,
    activation_map: ActivationMap,
    capsules_countdown: CapsulesCountdown,
    display_column: [Color; NUM_LINES],
    leader_color: Color,
    your_color: Color,
    opponent_color: Color,
    droid_num: u16,
    opponent_type: Droid,
    pub to_game_blocks: [Rect; NUM_TO_BLOCKS],
    pub to_ground_blocks: [Rect; NUM_GROUND_BLOCKS],
    pub column_block: Rect,
    pub leader_block: Rect,
    pub left_ground_start: Point,
    pub right_ground_start: Point,
    pub column_start: Point,
    pub leader_block_start: Point,
    pub leader_led: Rect,
    pub fill_block: Rect,
    pub element_rect: Rect,
    pub capsule_rect: Rect,
    pub ground_rect: Rect,
    pub column_rect: Rect,
    // the global surface containing all game-blocks
    pub to_blocks: Option<Surface<'sdl>>,
    // the rectangles containing the blocks
    pub fill_blocks: [Rect; NUM_FILL_BLOCKS],
    pub capsule_blocks: [Rect; NUM_CAPS_BLOCKS],
    pub left_capsule_starts: [Point; COLORS],
    pub cur_capsule_starts: [Point; COLORS],
    pub playground_starts: [Point; COLORS],
    pub droid_starts: [Point; COLORS],
    direction: i32,
    flicker_color: i32,
    // your energy if you're rejected
    reject_energy: i32,
}

impl Default for Takeover<'_> {
    fn default() -> Self {
        Self {
            capsule_cur_row: [0, 0],
            num_capsules: [0, 0],
            playground: [[[Block::Cable; NUM_LINES]; NUM_LAYERS]; COLORS].into(),
            activation_map: [[[Condition::Inactive; NUM_LINES]; NUM_LAYERS]; COLORS].into(),
            capsules_countdown: [[[None; NUM_LINES]; NUM_LAYERS]; COLORS].into(),
            display_column: [
                Color::Yellow,
                Color::Violet,
                Color::Yellow,
                Color::Violet,
                Color::Yellow,
                Color::Violet,
                Color::Yellow,
                Color::Violet,
                Color::Yellow,
                Color::Violet,
                Color::Yellow,
                Color::Violet,
            ],
            leader_color: Color::Yellow,
            your_color: Color::Yellow,
            opponent_color: Color::Violet,
            droid_num: 0,
            opponent_type: Droid::Droid001,
            to_game_blocks: [Rect::default(); NUM_TO_BLOCKS],
            to_ground_blocks: [Rect::default(); NUM_GROUND_BLOCKS],
            column_block: Rect::default(),
            leader_block: Rect::default(),
            left_ground_start: Point {
                x: 2 * 10,
                y: 2 * 15,
            },
            right_ground_start: Point {
                x: 2 * 255,
                y: 2 * 15,
            },
            column_start: Point {
                x: 2 * 136,
                y: 2 * 27,
            },
            leader_block_start: Point {
                x: 2 * 129,
                y: 2 * 8,
            },
            leader_led: Rect::new(2 * 136, 2 * 11, 2 * 16, 2 * 19),
            fill_block: Rect::new(0, 0, 2 * 16, 2 * 7),
            element_rect: Rect::new(0, 0, 2 * 32, 2 * 8),
            capsule_rect: Rect::new(0, 0, 2 * 7, 2 * 8),
            ground_rect: Rect::new(0, 0, 2 * 23, 2 * 8),
            column_rect: Rect::new(0, 0, 2 * 30, 2 * 8),
            to_blocks: None,
            fill_blocks: [Rect::default(); NUM_FILL_BLOCKS],
            capsule_blocks: [Rect::default(); NUM_CAPS_BLOCKS],
            left_capsule_starts: [
                Point { x: 4, y: 2 * 27 },
                Point {
                    x: 2 * 255 + 2 * 30 - 10,
                    y: 2 * 27,
                },
            ],
            cur_capsule_starts: [
                Point {
                    x: 2 * 26,
                    y: 2 * 19,
                },
                Point {
                    x: 2 * 255,
                    y: 2 * 19,
                },
            ],
            playground_starts: [
                Point {
                    x: 2 * 33,
                    y: 2 * 26,
                },
                Point {
                    x: 2 * 159,
                    y: 2 * 26,
                },
            ],
            droid_starts: [Point { x: 2 * 40, y: -4 }, Point { x: 2 * 220, y: -4 }],
            direction: 1,
            flicker_color: 0,
            reject_energy: 0,
        }
    }
}

/* File containing the Takeover-blocks */
pub const TO_BLOCK_FILE: &[u8] = b"to_elem.png";

/* --------------- individual block dimensions --------------- */
const NUM_PHASES: usize =		5       /* number of color-phases for current "flow" */;
/* inclusive "inactive" phase */

/* Dimensions of the game-blocks */
const TO_BLOCKS_N: usize = 11; /* anzahl versch. Game- blocks */

const NUM_TO_BLOCKS: usize = 2 * NUM_PHASES * TO_BLOCKS_N; // total number of takover blocks
const TO_ELEMENTS: u8 = 6;

/* Dimensions of the fill-blocks (in led-column */
const NUM_FILL_BLOCKS: usize = 3; // yellow, violet and black

/* Dimensions of a capsule */
const NUM_CAPS_BLOCKS: usize = 3; // yellow, violet and red (?what for)

/* Dimensions of ground-, column- und leader blocks */
const NUM_GROUND_BLOCKS: usize = 6;

/* --------------- Timing parameters --------------- */
const CAPSULE_COUNTDOWN: u8 = 40; /* 1/10 sec. Lebensdauer einer Kapsel */

/* --------------- Playground layout --------------- */

const NUM_LAYERS: usize = 4; /* dimension of the playground */
const NUM_LINES: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroundBlock {
    YellowAbove,
    YellowMiddle,
    YellowBelow,
    VioletAbove,
    VioletMiddle,
    VioletBelow,
}

/* Konditions in Connection-layer */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Condition {
    Inactive,
    Active1,
    Active2,
    Active3,
    Active4,
}

impl Condition {
    const fn is_active(self) -> bool {
        use Condition as C;
        match self {
            C::Inactive => false,
            C::Active1 | C::Active2 | C::Active3 | C::Active4 => true,
        }
    }

    const fn is_inactive(self) -> bool {
        !self.is_active()
    }

    fn next_active(self) -> Condition {
        use Condition as C;

        match self {
            C::Active1 => C::Active2,
            C::Active2 => C::Active3,
            C::Active3 => C::Active4,
            C::Active4 => C::Active1,
            C::Inactive => panic!("next_active called on inactive condition"),
        }
    }
}

impl From<Condition> for usize {
    fn from(condition: Condition) -> Self {
        use Condition as C;
        match condition {
            C::Inactive => 0,
            C::Active1 => 1,
            C::Active2 => 2,
            C::Active3 => 3,
            C::Active4 => 4,
        }
    }
}

/* Names for you and "him" */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Opponents {
    You,
    Enemy,
}

/* Color-names */
const COLORS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    Yellow = 0,
    Violet,
    Draw,
}

macro_rules! impl_try_from_to_color {
    ($($ty:ty),+) => {
        $(
            impl TryFrom<$ty> for Color {
                type Error = Infallible;

                fn try_from(value: $ty) -> Result<Self, Self::Error> {
                    Ok(match value {
                        0 => Color::Yellow,
                        1 => Color::Violet,
                        2 => Color::Draw,
                        _ => panic!("invalid raw color value"),
                    })
                }
            }
        )+
    }
}
impl_try_from_to_color!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);

impl From<Color> for usize {
    fn from(color: Color) -> Self {
        match color {
            Color::Yellow => 0,
            Color::Violet => 1,
            Color::Draw => 2,
        }
    }
}

/* Element - Names */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ToElement {
    Cable,
    CableEnd,
    Repeater,
    ColorSwapper,
    Branch,
    Gate,
}

impl TryFrom<u8> for ToElement {
    type Error = Infallible;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use ToElement as T;
        Ok(match value {
            0 => T::Cable,
            1 => T::CableEnd,
            2 => T::Repeater,
            3 => T::ColorSwapper,
            4 => T::Branch,
            5 => T::Gate,
            _ => panic!("invalid raw ToElement value"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Block {
    Cable,
    CableEnd,
    Repeater,
    ColorSwapper,
    BranchAbove,
    BranchMiddle,
    BranchBelow,
    GateAbove,
    GateMiddle,
    GateBelow,
    Empty,
}

impl Block {
    const fn is_connector(self) -> bool {
        use Block as B;
        match self {
            B::Cable
            | B::Repeater
            | B::ColorSwapper
            | B::BranchAbove
            | B::BranchBelow
            | B::GateMiddle => true,

            B::CableEnd | B::BranchMiddle | B::GateAbove | B::GateBelow | B::Empty => false,
        }
    }
}

impl From<Block> for usize {
    fn from(block: Block) -> Self {
        match block {
            Block::Cable => 0,
            Block::CableEnd => 1,
            Block::Repeater => 2,
            Block::ColorSwapper => 3,
            Block::BranchAbove => 4,
            Block::BranchMiddle => 5,
            Block::BranchBelow => 6,
            Block::GateAbove => 7,
            Block::GateMiddle => 8,
            Block::GateBelow => 9,
            Block::Empty => 10,
        }
    }
}

impl Takeover<'_> {
    fn process_playground(&mut self) {
        let Self {
            activation_map,
            playground,
            ..
        } = self;

        activation_map.iter_mut().zip(playground.iter()).for_each(
            |(activation_line, playground_line)| {
                playground_line
                    .iter()
                    .enumerate()
                    .skip(1)
                    .for_each(|(layer, playground_layer)| {
                        let (activation_layer_last, activation_layer) =
                            activation_line.split_at_mut(layer);
                        let activation_layer_last = activation_layer_last.last().unwrap();
                        let activation_layer = &mut activation_layer[0];

                        playground_layer
                            .iter()
                            .enumerate()
                            .for_each(|(row, &playground_block)| {
                                process_playground_row(
                                    row,
                                    playground_block,
                                    &**activation_layer_last,
                                    &mut **activation_layer,
                                );
                            });
                    });

                let [.., proximal_activation_layer, distal_activation_layer] =
                    activation_line.layers_mut();
                distal_activation_layer
                    .iter_mut()
                    .enumerate()
                    .for_each(|(row, activation)| {
                        let test_element = playground_line.proximal_connection()[row];
                        if proximal_activation_layer[row].is_active() && test_element.is_connector()
                        {
                            *activation = Condition::Active1;
                        } else {
                            *activation = Condition::Inactive;
                        }
                    });
            },
        );
    }
}

#[inline]
fn process_playground_row(
    row: usize,
    playground: Block,
    activation_layer_last: &[Condition],
    activation_layer: &mut [Condition],
) {
    use Block as B;

    let activation_last_layer = activation_layer_last[row];
    let (activation_last, activation_layer) = activation_layer.split_at_mut(row);
    let activation_last = activation_last.last().copied();
    let (activation, activation_layer) = activation_layer.split_first_mut().unwrap();
    let activation_next = activation_layer.first().copied();

    let turn_active = match playground {
        B::ColorSwapper | B::BranchMiddle | B::GateAbove | B::GateBelow | B::Cable => {
            activation_last_layer.is_active()
        }
        B::Repeater => activation_last_layer.is_active() || activation.is_active(),
        B::BranchAbove => activation_next.is_some_and(Condition::is_active),
        B::BranchBelow => activation_last.is_some_and(Condition::is_active),
        B::GateMiddle => {
            activation_last.is_some_and(Condition::is_active)
                && activation_next.is_some_and(Condition::is_active)
        }
        B::CableEnd | B::Empty => false,
    };

    if turn_active {
        if activation.is_inactive() {
            *activation = Condition::Active1;
        }
    } else {
        *activation = Condition::Inactive;
    }
}

/// Define all the Rects for the takeover-game
impl crate::Data<'_> {
    pub fn set_takeover_rects(&mut self) -> i32 {
        let Self {
            takeover:
                Takeover {
                    fill_blocks,
                    fill_block,
                    capsule_rect,
                    capsule_blocks,
                    ..
                },
            ..
        } = self;
        /* Set the fill-blocks */
        fill_blocks
            .iter_mut()
            .zip((0..).step_by(usize::from(fill_block.width()) + 2))
            .for_each(|(rect, cur_x)| {
                *rect = Rect::new(cur_x, 0, fill_block.width(), fill_block.height());
            });

        /* Set the capsule Blocks */
        let start_x = i16::try_from(self.takeover.fill_blocks.len()).unwrap()
            * (i16::try_from(self.takeover.fill_block.width()).unwrap() + 2);
        capsule_blocks
            .iter_mut()
            .zip((start_x..).step_by(usize::from(capsule_rect.width()) + 2))
            .for_each(|(rect, cur_x)| {
                *rect = Rect::new(cur_x, 0, capsule_rect.width(), capsule_rect.height() - 2);
            });

        /* get the game-blocks */
        let Self {
            takeover:
                Takeover {
                    to_game_blocks,
                    to_ground_blocks,
                    fill_block,
                    element_rect,
                    ground_rect,
                    ..
                },
            ..
        } = self;
        to_game_blocks
            .iter_mut()
            .zip(
                ((fill_block.height() + 2)..)
                    .step_by(usize::from(element_rect.height()) + 2)
                    .flat_map(|cur_y| {
                        (0..)
                            .step_by(usize::from(element_rect.width()) + 2)
                            .take(TO_BLOCKS_N)
                            .map(move |cur_x| (cur_x, cur_y))
                    }),
            )
            .for_each(|(rect, (cur_x, cur_y))| {
                *rect = Rect::new(
                    cur_x,
                    cur_y.try_into().unwrap(),
                    element_rect.width(),
                    element_rect.height(),
                );
            });
        let mut cur_y = (self.takeover.fill_block.height() + 2)
            + (self.takeover.element_rect.height() + 2) * u16::try_from(NUM_PHASES).unwrap() * 2;

        /* Get the ground, column and leader blocks */
        to_ground_blocks
            .iter_mut()
            .zip((0..).step_by(usize::from(ground_rect.width()) + 2))
            .for_each(|(rect, cur_x)| {
                *rect = Rect::new(
                    cur_x,
                    cur_y.try_into().unwrap(),
                    ground_rect.width(),
                    ground_rect.height(),
                );
            });
        cur_y += self.takeover.ground_rect.height() + 2;
        self.takeover.column_block = Rect::new(
            0,
            cur_y.try_into().unwrap(),
            self.takeover.column_rect.width(),
            self.takeover.column_rect.height(),
        );
        self.takeover.leader_block = Rect::new(
            i16::try_from(self.takeover.column_rect.width()).unwrap() + 2,
            cur_y.try_into().unwrap(),
            self.takeover.leader_led.width() * 2 - 4,
            self.takeover.leader_led.height(),
        );
        defs::OK.into()
    }

    fn enemy_movements(&mut self) {
        const MOVE_PROBABILITY: i32 = 100;
        const TURN_PROBABILITY: i32 = 10;
        const SET_PROBABILITY: i32 = 80;

        enum Action {
            Move,
            Turn,
            SetCapsule,
            Nothing,
        }

        let opponent_color = self.takeover.opponent_color as usize;
        let mut row = self.takeover.capsule_cur_row[opponent_color] - 1;

        if self.takeover.num_capsules[Opponents::Enemy as usize] == 0 {
            return;
        }

        let mut rng = thread_rng();
        let next_row = match [
            Action::Move,
            Action::Turn,
            Action::SetCapsule,
            Action::Nothing,
        ]
        .choose(&mut rng)
        .unwrap()
        {
            Action::Move => {
                if (0..=100).choose(&mut rng).unwrap() <= MOVE_PROBABILITY {
                    row += self.takeover.direction;
                    if row > i32::try_from(NUM_LINES).unwrap() - 1 {
                        1
                    } else if row < 0 {
                        i32::try_from(NUM_LINES).unwrap()
                    } else {
                        row + 1
                    }
                } else {
                    row + 1
                }
            }

            Action::Turn => {
                /* Turn around */
                if (0..=100).choose(&mut rng).unwrap() <= TURN_PROBABILITY {
                    self.takeover.direction *= -1;
                }
                row + 1
            }

            Action::SetCapsule => {
                /* Try to set  capsule */
                match usize::try_from(row) {
                    Ok(row)
                        if (0..=100).choose(&mut rng).unwrap() <= SET_PROBABILITY
                            && self.takeover.playground[opponent_color][0][row]
                                != Block::CableEnd
                            && self.takeover.activation_map[opponent_color][0][row]
                                == Condition::Inactive =>
                    {
                        self.takeover.num_capsules[Opponents::Enemy as usize] -= 1;
                        self.takeover_set_capsule_sound();
                        self.takeover.playground[opponent_color][0][row] = Block::Repeater;
                        self.takeover.activation_map[opponent_color][0][row] = Condition::Active1;
                        self.takeover.capsules_countdown[opponent_color][0][row] =
                            Some(CAPSULE_COUNTDOWN * 2);
                        0
                    }
                    _ => row + 1,
                }
            }

            Action::Nothing => row + 1,
        };

        self.takeover.capsule_cur_row[opponent_color] = next_row;
    }

    /// Animate the active cables: this is done by cycling over
    /// the active phases ACTIVE1-ACTIVE3, which are represented by
    /// different pictures in the playground
    fn animate_currents(&mut self) {
        self.takeover
            .activation_map
            .iter_mut()
            .flat_map(map::Line::iter_mut)
            .flat_map(map::Layer::iter_mut)
            .filter(|condition| condition.is_active())
            .for_each(|condition| *condition = condition.next_active());
    }

    /// does the countdown of the capsules and kills them if too old
    fn process_capsules(&mut self) {
        self.takeover
            .capsules_countdown
            .iter_mut()
            .flat_map(|color_countdown| color_countdown[0].iter_mut())
            .zip(
                self.takeover
                    .activation_map
                    .iter_mut()
                    .flat_map(|color_activation| color_activation[0].iter_mut()),
            )
            .zip(
                self.takeover
                    .playground
                    .iter_mut()
                    .flat_map(|color_playground| color_playground[0].iter_mut()),
            )
            .for_each(|((countdown, activation), playground)| {
                if let Some(count) = countdown.as_mut() {
                    *count = count.saturating_sub(1);

                    if *count == 0 {
                        *countdown = None;
                        *activation = Condition::Inactive;
                        *playground = Block::Cable;
                    }
                }
            });
    }

    fn process_display_column(&mut self) {
        use std::cmp::Ordering;

        const CONNECTION_LAYER: usize = 3;

        self.takeover.flicker_color = !self.takeover.flicker_color;

        let Self {
            takeover:
                Takeover {
                    activation_map,
                    playground,
                    display_column,
                    ref flicker_color,
                    ..
                },
            ..
        } = self;

        activation_map[Color::Yellow as usize][CONNECTION_LAYER]
            .iter()
            .zip(activation_map[Color::Violet as usize][CONNECTION_LAYER].iter())
            .zip(playground[Color::Yellow as usize][CONNECTION_LAYER - 1].iter())
            .zip(playground[Color::Violet as usize][CONNECTION_LAYER - 1].iter())
            .zip(display_column.iter_mut())
            .for_each(
                |(
                    (
                        ((&yellow_activation, &violet_activation), &yellow_playground),
                        &violet_playground,
                    ),
                    display,
                )| {
                    if yellow_activation.is_active() && violet_activation.is_inactive() {
                        if yellow_playground == Block::ColorSwapper {
                            *display = Color::Violet;
                        } else {
                            *display = Color::Yellow;
                        }
                    } else if yellow_activation.is_inactive() && violet_activation.is_active() {
                        if violet_playground == Block::ColorSwapper {
                            *display = Color::Yellow;
                        } else {
                            *display = Color::Violet;
                        }
                    } else if yellow_activation.is_active() && violet_activation.is_active() {
                        if yellow_playground == Block::ColorSwapper
                            && violet_playground != Block::ColorSwapper
                        {
                            *display = Color::Violet;
                        } else if (yellow_playground != Block::ColorSwapper
                            && violet_playground == Block::ColorSwapper)
                            || *flicker_color == 0
                        {
                            *display = Color::Yellow;
                        } else {
                            *display = Color::Violet;
                        }
                    }
                },
            );

        let mut yellow_counter = 0;
        let mut violet_counter = 0;
        for &color in display_column.iter() {
            if color == Color::Yellow {
                yellow_counter += 1;
            } else {
                violet_counter += 1;
            }
        }

        match violet_counter.cmp(&yellow_counter) {
            Ordering::Less => self.takeover.leader_color = Color::Yellow,
            Ordering::Greater => self.takeover.leader_color = Color::Violet,
            Ordering::Equal => self.takeover.leader_color = Color::Draw,
        }
    }

    /// process the playground following its intrinsic logic
    fn process_playground(&mut self) {
        self.takeover.process_playground();
    }

    /// generate a random Playground
    fn invent_playground(&mut self) {
        const MAX_PROB: i32 = 100;
        const ELEMENTS_PROBABILITIES: [i32; u8_to_usize(TO_ELEMENTS)] = [
            100, /* Cable */
            2,   /* CableEnd */
            5,   /* Repeater */
            5,   /* ColorSwapper: only on last layer */
            5,   /* Branch */
            5,   /* Gate */
        ];

        /* first clear the playground: we depend on this !! */
        self.clear_playground();

        let mut rng = thread_rng();
        self.takeover
            .playground
            .iter_mut()
            .for_each(|playground_color| {
                for layer in 1..NUM_LAYERS {
                    let (playground_prev_layers, playground_layer) =
                        playground_color.split_at_mut(layer);
                    let playground_prev_layer = playground_prev_layers.last_mut().unwrap();
                    let playground_layer = &mut playground_layer[0];

                    let mut row = 0;
                    while row < NUM_LINES {
                        let block = &mut playground_layer[row];
                        if !matches!(block, Block::Cable) {
                            row += 1;
                            continue;
                        }

                        let new_element = (0..TO_ELEMENTS).choose(&mut rng).unwrap();
                        if (0..=MAX_PROB).choose(&mut rng).unwrap()
                            > ELEMENTS_PROBABILITIES[usize::from(new_element)]
                        {
                            continue;
                        }

                        try_set_new_playground_element(
                            new_element,
                            &mut row,
                            layer,
                            playground_layer,
                            playground_prev_layer,
                        );
                    }
                }
            });
    }

    /// Clears Playground (and `self.takeover.activation_map`) to default start-values
    fn clear_playground(&mut self) {
        self.takeover
            .activation_map
            .iter_mut()
            .flatten()
            .flatten()
            .for_each(|activation| *activation = Condition::Inactive);

        self.takeover
            .playground
            .iter_mut()
            .flatten()
            .flatten()
            .for_each(|block| *block = Block::Cable);

        self.takeover
            .display_column
            .iter_mut()
            .enumerate()
            .for_each(|(row, display_column)| *display_column = (row % 2).try_into().unwrap());
    }

    /// prepares _and displays_ the current Playground
    ///
    /// NOTE: this function should only change the USERFENSTER part
    ///       so that we can do Infoline-setting before this
    fn show_playground(&mut self) {
        let your_color: usize = self.takeover.your_color.into();
        let opponent_color: usize = self.takeover.opponent_color.into();

        let x_offs = self.vars.classic_user_rect.x();
        let y_offs = self.vars.classic_user_rect.y();

        let Self {
            graphics:
                Graphics {
                    takeover_bg_pic,
                    ne_screen,
                    ..
                },
            vars,
            ..
        } = self;
        ne_screen.as_mut().unwrap().clear_clip_rect();

        let mut user_rect = vars.user_rect;
        takeover_bg_pic.as_mut().unwrap().blit_from_to(
            &vars.user_rect,
            ne_screen.as_mut().unwrap(),
            &mut user_rect,
        );
        vars.user_rect = user_rect;

        self.put_influence(
            i32::from(x_offs) + self.takeover.droid_starts[your_color].x,
            i32::from(y_offs) + self.takeover.droid_starts[your_color].y,
        );

        if self.main.enemys[usize::from(self.takeover.droid_num)].status != Status::Out {
            self.put_enemy(
                self.takeover.droid_num,
                i32::from(x_offs) + self.takeover.droid_starts[opponent_color].x,
                i32::from(y_offs) + self.takeover.droid_starts[opponent_color].y,
            );
        }

        self.show_yellow_blocks();
        self.show_violet_blocks();

        let Self {
            takeover:
                Takeover {
                    to_blocks,
                    display_column,
                    column_start,
                    column_rect,
                    fill_blocks,
                    ..
                },
            graphics: Graphics { ne_screen, .. },
            ..
        } = self;
        /* Fill the Display Column with its leds */
        display_column
            .iter()
            .copied()
            .enumerate()
            .for_each(|(line, display_column)| {
                let mut dst = Rect::new(
                    x_offs + i16::try_from(column_start.x).unwrap(),
                    y_offs
                        + i16::try_from(column_start.y).unwrap()
                        + i16::try_from(line).unwrap()
                            * i16::try_from(column_rect.height()).unwrap(),
                    0,
                    0,
                );
                to_blocks.as_mut().unwrap().blit_from_to(
                    &fill_blocks[usize::from(display_column)],
                    ne_screen.as_mut().unwrap(),
                    &mut dst,
                );
            });

        self.show_yellow_playground();
        self.show_violet_playground();
        self.show_capsules_left();
    }

    fn show_yellow_blocks(&mut self) {
        let Self {
            takeover:
                Takeover {
                    to_blocks,
                    leader_block,
                    to_ground_blocks,
                    column_block,
                    ..
                },
            graphics: Graphics { ne_screen, .. },
            ..
        } = self;

        let x_offs = self.vars.classic_user_rect.x();
        let y_offs = self.vars.classic_user_rect.y();

        let mut dst = Rect::new(
            x_offs + i16::try_from(self.takeover.left_ground_start.x).unwrap(),
            y_offs + i16::try_from(self.takeover.left_ground_start.y).unwrap(),
            self.vars.user_rect.width(),
            self.vars.user_rect.height(),
        );

        to_blocks.as_mut().unwrap().blit_from_to(
            &to_ground_blocks[GroundBlock::YellowAbove as usize],
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );

        dst.inc_y(i16::try_from(self.takeover.ground_rect.height()).unwrap());

        for _ in 0..12 {
            to_blocks.as_mut().unwrap().blit_from_to(
                &to_ground_blocks[GroundBlock::YellowMiddle as usize],
                ne_screen.as_mut().unwrap(),
                &mut dst,
            );

            dst.inc_y(i16::try_from(self.takeover.ground_rect.height()).unwrap());
        }

        to_blocks.as_mut().unwrap().blit_from_to(
            &to_ground_blocks[GroundBlock::YellowBelow as usize],
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );

        dst = Rect::new(
            x_offs + i16::try_from(self.takeover.leader_block_start.x).unwrap(),
            y_offs + i16::try_from(self.takeover.leader_block_start.y).unwrap(),
            0,
            0,
        );
        to_blocks.as_mut().unwrap().blit_from_to(
            &*leader_block,
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );

        dst.inc_y(i16::try_from(self.takeover.leader_led.height()).unwrap());
        for _ in 0..12 {
            to_blocks.as_mut().unwrap().blit_from_to(
                &*column_block,
                ne_screen.as_mut().unwrap(),
                &mut dst,
            );
            dst.inc_y(i16::try_from(self.takeover.column_rect.height()).unwrap());
        }
    }

    fn show_violet_blocks(&mut self) {
        let Self {
            takeover:
                Takeover {
                    to_blocks,
                    to_ground_blocks,
                    fill_blocks,
                    ..
                },
            graphics: Graphics { ne_screen, .. },
            ..
        } = self;

        let x_offs = self.vars.classic_user_rect.x();
        let y_offs = self.vars.classic_user_rect.y();

        let mut dst = Rect::new(
            x_offs + i16::try_from(self.takeover.right_ground_start.x).unwrap(),
            y_offs + i16::try_from(self.takeover.right_ground_start.y).unwrap(),
            0,
            0,
        );

        to_blocks.as_mut().unwrap().blit_from_to(
            &to_ground_blocks[GroundBlock::VioletAbove as usize],
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );
        dst.inc_y(i16::try_from(self.takeover.ground_rect.height()).unwrap());

        for _ in 0..12 {
            to_blocks.as_mut().unwrap().blit_from_to(
                &to_ground_blocks[GroundBlock::VioletMiddle as usize],
                ne_screen.as_mut().unwrap(),
                &mut dst,
            );
            dst.inc_y(i16::try_from(self.takeover.ground_rect.height()).unwrap());
        }

        to_blocks.as_mut().unwrap().blit_from_to(
            &to_ground_blocks[GroundBlock::VioletBelow as usize],
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );

        /* Fill the Leader-LED with its color */
        let leader_color = usize::from(self.takeover.leader_color);
        dst = Rect::new(
            x_offs + self.takeover.leader_led.x(),
            y_offs + self.takeover.leader_led.y(),
            0,
            0,
        );
        to_blocks.as_mut().unwrap().blit_from_to(
            &fill_blocks[leader_color],
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );
        dst.inc_y(i16::try_from(self.takeover.fill_block.height()).unwrap());
        to_blocks.as_mut().unwrap().blit_from_to(
            &fill_blocks[leader_color],
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );
    }

    fn show_yellow_playground(&mut self) {
        let Self {
            takeover:
                Takeover {
                    playground,
                    activation_map,
                    to_game_blocks,
                    element_rect,
                    to_blocks,
                    playground_starts,
                    ..
                },
            graphics: Graphics { ne_screen, .. },
            vars,
            ..
        } = self;

        let x_offs = vars.classic_user_rect.x();
        let y_offs = vars.classic_user_rect.y();

        playground[Color::Yellow as usize]
            .iter()
            .take(NUM_LAYERS - 1)
            .zip(
                activation_map[Color::Yellow as usize]
                    .iter()
                    .take(NUM_LAYERS - 1),
            )
            .enumerate()
            .flat_map(|(layer_index, (playground_layer, activation_layer))| {
                let layer_index = i16::try_from(layer_index).unwrap();
                playground_layer
                    .iter()
                    .copied()
                    .zip(activation_layer.iter().copied())
                    .enumerate()
                    .map(move |(line_index, (playground_line, activation_line))| {
                        (
                            layer_index,
                            i16::try_from(line_index).unwrap(),
                            usize::from(playground_line),
                            usize::from(activation_line),
                        )
                    })
            })
            .for_each(
                |(layer_index, line_index, playground_line, activation_line)| {
                    let mut dst = Rect::new(
                        x_offs
                            + i16::try_from(playground_starts[Color::Yellow as usize].x).unwrap()
                            + layer_index * i16::try_from(element_rect.width()).unwrap(),
                        y_offs
                            + i16::try_from(playground_starts[Color::Yellow as usize].y).unwrap()
                            + line_index * i16::try_from(element_rect.height()).unwrap(),
                        0,
                        0,
                    );

                    let block = playground_line + activation_line * TO_BLOCKS_N;
                    to_blocks.as_mut().unwrap().blit_from_to(
                        &to_game_blocks[block],
                        ne_screen.as_mut().unwrap(),
                        &mut dst,
                    );
                },
            );
    }

    fn show_violet_playground(&mut self) {
        let Self {
            takeover:
                Takeover {
                    playground,
                    activation_map,
                    to_game_blocks,
                    element_rect,
                    to_blocks,
                    playground_starts,
                    ..
                },
            graphics: Graphics { ne_screen, .. },
            vars,
            ..
        } = self;

        let x_offs = vars.classic_user_rect.x();
        let y_offs = vars.classic_user_rect.y();

        playground[Color::Violet as usize]
            .iter()
            .take(NUM_LAYERS - 1)
            .zip(
                activation_map[Color::Violet as usize]
                    .iter()
                    .take(NUM_LAYERS - 1),
            )
            .enumerate()
            .flat_map(|(layer_index, (playground_layer, activation_layer))| {
                let layer_index = i16::try_from(layer_index).unwrap();
                playground_layer
                    .iter()
                    .copied()
                    .zip(activation_layer.iter().copied())
                    .enumerate()
                    .map(move |(line_index, (playground_line, activation_line))| {
                        (
                            layer_index,
                            i16::try_from(line_index).unwrap(),
                            usize::from(playground_line),
                            usize::from(activation_line),
                        )
                    })
            })
            .for_each(
                |(layer_index, line_index, playground_line, activation_line)| {
                    let mut dst = Rect::new(
                        x_offs
                            + i16::try_from(playground_starts[Color::Violet as usize].x).unwrap()
                            + (i16::try_from(NUM_LAYERS).unwrap() - layer_index - 2)
                                * i16::try_from(element_rect.width()).unwrap(),
                        y_offs
                            + i16::try_from(playground_starts[Color::Violet as usize].y).unwrap()
                            + line_index * i16::try_from(element_rect.height()).unwrap(),
                        0,
                        0,
                    );
                    let block = playground_line + (NUM_PHASES + activation_line) * TO_BLOCKS_N;
                    to_blocks.as_mut().unwrap().blit_from_to(
                        &to_game_blocks[block],
                        ne_screen.as_mut().unwrap(),
                        &mut dst,
                    );
                },
            );
    }

    fn show_capsules_left(&mut self) {
        let Self {
            takeover:
                Takeover {
                    to_blocks,
                    num_capsules,
                    capsule_cur_row,
                    capsule_rect,
                    capsule_blocks,
                    left_capsule_starts,
                    cur_capsule_starts,
                    your_color,
                    opponent_color,
                    ..
                },
            graphics: Graphics { ne_screen, .. },
            vars,
            ..
        } = self;

        let your_color: usize = (*your_color).into();
        let opponent_color: usize = (*opponent_color).into();

        let x_offs = vars.classic_user_rect.x();
        let y_offs = vars.classic_user_rect.y();

        num_capsules
            .iter()
            .copied()
            .enumerate()
            .for_each(|(player, capsules)| {
                let color = if player == Opponents::You as usize {
                    your_color
                } else {
                    opponent_color
                };

                let mut dst = Rect::new(
                    x_offs + i16::try_from(cur_capsule_starts[color].x).unwrap(),
                    y_offs
                        + i16::try_from(cur_capsule_starts[color].y).unwrap()
                        + i16::try_from(capsule_cur_row[color]).unwrap()
                            * i16::try_from(capsule_rect.height()).unwrap(),
                    0,
                    0,
                );
                if capsules != 0 {
                    to_blocks.as_mut().unwrap().blit_from_to(
                        &capsule_blocks[color],
                        ne_screen.as_mut().unwrap(),
                        &mut dst,
                    );
                }

                for capsule in 0..capsules.saturating_sub(1) {
                    dst = Rect::new(
                        x_offs + i16::try_from(left_capsule_starts[color].x).unwrap(),
                        y_offs
                            + i16::try_from(left_capsule_starts[color].y).unwrap()
                            + i16::try_from(capsule).unwrap()
                                * i16::try_from(capsule_rect.height()).unwrap(),
                        0,
                        0,
                    );
                    to_blocks.as_mut().unwrap().blit_from_to(
                        &capsule_blocks[color],
                        ne_screen.as_mut().unwrap(),
                        &mut dst,
                    );
                }
            });
    }

    /// the acutal Takeover game-playing is done here
    fn play_game(&mut self) {
        const COUNT_TICK_LEN: u32 = 100;

        let mut countdown = 100;

        let mut prev_count_tick = self.sdl.ticks_ms();
        let mut prev_move_tick = prev_count_tick;

        self.wait_for_all_keys_released();

        self.countdown_sound();
        let mut count_text = ArrayCString::<11>::default();

        loop {
            match self.play_takeover_once(
                &mut countdown,
                &mut count_text,
                &mut prev_count_tick,
                &mut prev_move_tick,
            ) {
                PlayTakeoverOnce::Continue => {}
                PlayTakeoverOnce::Finish => break,
                PlayTakeoverOnce::Return => return,
            }
        }

        /* Schluss- Countdown */
        countdown = CAPSULE_COUNTDOWN;

        self.wait_for_all_keys_released();
        let mut fast_forward = false;
        loop {
            countdown -= 1;
            if countdown == 0 {
                break;
            }

            if !fast_forward {
                self.sdl.delay_ms(COUNT_TICK_LEN);
            }
            if self.any_key_just_pressed() != 0 {
                fast_forward = true;
            }
            prev_count_tick += COUNT_TICK_LEN;
            self.process_capsules(); /* count down the lifetime of the capsules */
            self.process_capsules(); /* do it twice this time to be faster */
            self.animate_currents();
            /* this has to be done several times to be sure */
            for _ in 0..4 {
                self.process_playground();
            }
            self.process_display_column();
            self.show_playground();
            self.sdl.delay_ms(1);
            assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
        } /* while (countdown) */

        self.wait_for_all_keys_released();
    }

    fn play_takeover_once(
        &mut self,
        countdown: &mut u8,
        count_text: &mut ArrayCString<11>,
        prev_count_tick: &mut u32,
        prev_move_tick: &mut u32,
    ) -> PlayTakeoverOnce {
        const COUNT_TICK_LEN: u32 = 100;
        const MOVE_TICK_LEN: u32 = 60;

        let your_color = usize::from(self.takeover.your_color);
        let mut outcome = PlayTakeoverOnce::Continue;
        let cur_time = self.sdl.ticks_ms();

        let do_update_count = cur_time > *prev_count_tick + COUNT_TICK_LEN;
        if do_update_count {
            use std::fmt::Write;

            /* time to count 1 down */
            *prev_count_tick += COUNT_TICK_LEN; /* set for next countdown tick */
            *countdown -= 1;
            count_text.clear();
            write!(count_text, "Finish-{countdown}").unwrap();
            self.display_banner(Some(&*count_text), None, 0);

            if *countdown != 0 && *countdown % 10 == 0 {
                self.countdown_sound();
            }
            if *countdown == 0 {
                self.end_countdown_sound();
                outcome = PlayTakeoverOnce::Finish;
            }

            self.animate_currents(); /* do some animation on the active cables */
        }

        let do_update_move = cur_time > *prev_move_tick + MOVE_TICK_LEN;
        if do_update_move {
            *prev_move_tick += MOVE_TICK_LEN; /* set for next motion tick */

            let key_repeat_delay = if cfg!(target_os = "android") {
                150 // better to avoid accidential key-repeats on touchscreen
            } else {
                110 // PC default, allows for quick-repeat key hits
            };

            let action = self.get_menu_action(key_repeat_delay);
            /* allow for a WIN-key that give immedate victory */
            if self.key_is_pressed_r(b'w'.into()) && self.ctrl_pressed() && self.alt_pressed() {
                self.takeover.leader_color = self.takeover.your_color; /* simple as that */
                return PlayTakeoverOnce::Return;
            }

            if action.intersects(MenuAction::UP | MenuAction::UP_WHEEL) {
                self.takeover.capsule_cur_row[your_color] -= 1;
                if self.takeover.capsule_cur_row[your_color] < 1 {
                    self.takeover.capsule_cur_row[your_color] = NUM_LINES.try_into().unwrap();
                }
            }

            if action.intersects(MenuAction::DOWN | MenuAction::DOWN_WHEEL) {
                self.takeover.capsule_cur_row[your_color] += 1;
                if self.takeover.capsule_cur_row[your_color] > NUM_LINES.try_into().unwrap() {
                    self.takeover.capsule_cur_row[your_color] = 1;
                }
            }

            if action.intersects(MenuAction::CLICK) {
                if let Ok(row) = usize::try_from(self.takeover.capsule_cur_row[your_color] - 1) {
                    if self.takeover.num_capsules[Opponents::You as usize] > 0
                        && self.takeover.playground[your_color][0][row] != Block::CableEnd
                        && self.takeover.activation_map[your_color][0][row] == Condition::Inactive
                    {
                        self.takeover.num_capsules[Opponents::You as usize] -= 1;
                        self.takeover.capsule_cur_row[your_color] = 0;
                        self.takeover.playground[your_color][0][row] = Block::Repeater;
                        self.takeover.activation_map[your_color][0][row] = Condition::Active1;
                        self.takeover.capsules_countdown[your_color][0][row] =
                            Some(CAPSULE_COUNTDOWN * 2);
                        self.takeover_set_capsule_sound();
                    }
                }
            }

            self.enemy_movements();
            self.process_capsules(); /* count down the lifetime of the capsules */

            /* this has to be done several times to be sure */
            for _ in 0..4 {
                self.process_playground();
            }

            self.process_display_column();
            self.show_playground();
        } // if do_update_move

        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
        self.sdl.delay_ms(1);

        outcome
    }

    fn choose_color(&mut self) {
        const COUNT_TICK_LEN: u32 = 100; /* countdown in 1/10 second steps */

        let mut countdown = 100; /* duration in 1/10 seconds given for color choosing */

        let mut prev_count_tick = self.sdl.ticks_ms();

        self.wait_for_all_keys_released();

        let mut color_chosen = false;
        let mut count_text = ArrayCString::<10>::default();
        while !color_chosen {
            let action = self.get_menu_action(110);
            if action.intersects(MenuAction::RIGHT | MenuAction::DOWN_WHEEL) {
                if self.takeover.your_color != Color::Violet {
                    self.move_menu_position_sound();
                }
                self.takeover.your_color = Color::Violet;
                self.takeover.opponent_color = Color::Yellow;
            }

            if action.intersects(MenuAction::LEFT | MenuAction::UP_WHEEL) {
                if self.takeover.your_color != Color::Yellow {
                    self.move_menu_position_sound();
                }
                self.takeover.your_color = Color::Yellow;
                self.takeover.opponent_color = Color::Violet;
            }

            if action.intersects(MenuAction::CLICK) {
                color_chosen = true;
            }

            /* wait for next countdown tick */
            if self.sdl.ticks_ms() >= prev_count_tick + COUNT_TICK_LEN {
                use std::fmt::Write;

                prev_count_tick += COUNT_TICK_LEN; /* set for next tick */
                countdown -= 1; /* Count down */
                count_text.clear();
                write!(count_text, "Color-{countdown}").unwrap();

                self.display_banner(Some(&*count_text), None, 0);
                self.show_playground();
            }

            if countdown == 0 {
                color_chosen = true;
            }

            assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
            self.sdl.delay_ms(1); // don't hog CPU
        }
    }

    /// play takeover-game against a druid
    ///
    /// Returns true if the user won, false otherwise
    pub fn takeover(&mut self, enemynum: u16) -> i32 {
        const BG_COLOR: SDL_Color = SDL_Color {
            r: 130,
            g: 130,
            b: 130,
            unused: 0,
        };

        /* Prevent distortion of framerate by the delay coming from
         * the time spend in the menu.
         */
        self.activate_conservative_frame_computation();

        // Takeover game always uses Classic User_Rect:
        let buf = self.vars.user_rect;
        self.vars.user_rect = self.vars.classic_user_rect;

        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());

        self.fill_rect(self.vars.user_rect, BG_COLOR);

        self.vars.me.status = Status::Mobile; /* the new status _after_ the takeover game */

        self.sdl.cursor().hide(); // no mouse-cursor in takeover game!

        self.show_droid_info(self.vars.me.ty, -1, 0);
        self.show_droid_portrait(
            self.vars.cons_droid_rect,
            self.vars.me.ty,
            DROID_ROTATION_TIME,
            UPDATE,
        );

        self.wait_for_all_keys_released();
        while !self.fire_pressed_r() {
            self.show_droid_portrait(
                self.vars.cons_droid_rect,
                self.vars.me.ty,
                DROID_ROTATION_TIME,
                0,
            );
            self.sdl.delay_ms(1);
        }

        let enemy_index: usize = enemynum.into();
        let enemy_type = self.main.enemys[enemy_index].ty;
        self.show_droid_info(enemy_type, -2, 0);
        self.show_droid_portrait(
            self.vars.cons_droid_rect,
            enemy_type,
            DROID_ROTATION_TIME,
            UPDATE,
        );
        self.wait_for_all_keys_released();
        while !self.fire_pressed_r() {
            self.show_droid_portrait(
                self.vars.cons_droid_rect,
                enemy_type,
                DROID_ROTATION_TIME,
                0,
            );
            self.sdl.delay_ms(1);
        }

        let Graphics {
            takeover_bg_pic,
            ne_screen,
            ..
        } = &mut self.graphics;
        takeover_bg_pic
            .as_mut()
            .unwrap()
            .blit(ne_screen.as_mut().unwrap());
        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());

        self.wait_for_all_keys_released();
        let mut finish_takeover = false;
        while !finish_takeover {
            self.takeover_round(enemynum, enemy_index, &mut finish_takeover);
        }

        // restore User_Rect
        self.vars.user_rect = buf;

        self.clear_graph_mem();

        (self.takeover.leader_color == self.takeover.your_color).into()
    }

    fn takeover_round(&mut self, enemynum: u16, enemy_index: usize, finish_takeover: &mut bool) {
        let enemy = &self.main.enemys[enemy_index];

        /* Init Color-column and Capsule-Number for each opponenet and your color */
        self.takeover
            .display_column
            .iter_mut()
            .enumerate()
            .for_each(|(row, column)| *column = (row % 2).try_into().unwrap());
        self.takeover
            .capsules_countdown
            .iter_mut()
            .flat_map(|color_countdown| color_countdown[0].iter_mut())
            .for_each(|x| *x = None);

        self.takeover.your_color = Color::Yellow;
        self.takeover.opponent_color = Color::Violet;

        self.takeover.capsule_cur_row[usize::from(Color::Yellow)] = 0;
        self.takeover.capsule_cur_row[usize::from(Color::Violet)] = 0;

        self.takeover.droid_num = enemynum;
        self.takeover.opponent_type = enemy.ty;
        self.takeover.num_capsules[Opponents::You as usize] =
            3 + self.class_of_druid(self.vars.me.ty);
        self.takeover.num_capsules[Opponents::Enemy as usize] =
            4 + self.class_of_druid(self.takeover.opponent_type);

        self.invent_playground();

        self.show_playground();
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

        self.choose_color();
        self.wait_for_all_keys_released();

        self.play_game();
        self.wait_for_all_keys_released();

        let message;
        /* Ausgang beurteilen und returnen */
        if self.main.invincible_mode != 0 || self.takeover.leader_color == self.takeover.your_color
        {
            message = self.takeover_win(enemy_index, finish_takeover);
        } else if self.takeover.leader_color == self.takeover.opponent_color {
            /* self.takeover.leader_color == self.takeover.your_color */
            // you lost, but enemy is killed too --> blast it!
            self.main.enemys[enemy_index].energy = -1.0; /* to be sure */

            self.takeover_game_lost_sound();
            #[allow(clippy::cast_precision_loss)]
            if self.vars.me.ty == Droid::Droid001 {
                message = cstr!("Burnt Out");
                self.vars.me.energy = 0.;
            } else {
                message = cstr!("Rejected");
                self.vars.me.ty = Droid::Droid001;
                self.vars.me.energy = self.takeover.reject_energy as f32;
            }
            *finish_takeover = true;
        } else {
            /* LeadColor == self.takeover.opponent_color */

            self.takeover_game_deadlock_sound();
            message = cstr!("Deadlock");
        }

        self.display_banner(Some(message), None, 0);
        self.show_playground();
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

        self.wait_for_all_keys_released();
        let now = self.sdl.ticks_ms();
        while !self.fire_pressed_r() && self.sdl.ticks_ms() - now < SHOW_WAIT {
            #[cfg(target_os = "android")]
            assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

            self.sdl.delay_ms(1);
        }
    }

    fn takeover_win(&mut self, enemy_index: usize, finish_takeover: &mut bool) -> &'static CStr {
        self.takeover_game_won_sound();
        #[allow(clippy::cast_possible_truncation)]
        if self.vars.me.ty == Droid::Droid001 {
            self.takeover.reject_energy = self.vars.me.energy as i32;
            self.main.pre_take_energy = self.vars.me.energy as i32;
        }

        // We provide some security agains too high energy/health values gained
        // by very rapid successions of successful takeover attempts
        let droid_map = &self.vars.droidmap;
        if self.vars.me.energy > droid_map[Droid::Droid001 as usize].maxenergy {
            self.vars.me.energy = droid_map[Droid::Droid001 as usize].maxenergy;
        }
        if self.vars.me.health > droid_map[Droid::Droid001 as usize].maxenergy {
            self.vars.me.health = droid_map[Droid::Droid001 as usize].maxenergy;
        }

        let enemy = &mut self.main.enemys[enemy_index];

        // We allow to gain the current energy/full health that was still in the
        // other droid, since all previous damage must be due to fighting damage,
        // and this is exactly the sort of damage can usually be cured in refreshes.
        self.vars.me.energy += enemy.energy;
        self.vars.me.health += droid_map[self.takeover.opponent_type.to_usize()].maxenergy;

        self.vars.me.ty = enemy.ty;

        #[allow(clippy::cast_precision_loss)]
        {
            self.main.real_score += droid_map[self.takeover.opponent_type.to_usize()].score as f32;

            self.main.death_count += f32::from(self.takeover.opponent_type.to_u16().pow(2));
            // quadratic "importance", max=529
        }

        enemy.status = Status::Out; // removed droid silently (no blast!)

        let message = if self.takeover.leader_color == self.takeover.your_color {
            /* won the proper way */
            cstr!("Complete")
        } else {
            /* only won because of InvincibleMode */
            cstr!("You cheat")
        };

        *finish_takeover = true;
        message
    }
}

#[inline]
fn try_set_new_playground_element(
    new_element: u8,
    row: &mut usize,
    layer: usize,
    playground_layer: &mut map::Layer<Block>,
    playground_prev_layer: &mut map::Layer<Block>,
) {
    fn cut_cable(block: &mut Block) {
        if block.is_connector() {
            *block = Block::CableEnd;
        }
    }

    let block = &mut playground_layer[*row];
    let prev_block = playground_prev_layer[*row];
    match ToElement::try_from(new_element).unwrap() {
        ToElement::Cable => {
            if prev_block.is_connector().not() {
                *block = Block::Empty;
            }
        }
        ToElement::CableEnd => {
            if prev_block.is_connector() {
                *block = Block::CableEnd;
            } else {
                *block = Block::Empty;
            }
        }
        ToElement::Repeater => {
            if prev_block.is_connector() {
                *block = Block::Repeater;
            } else {
                *block = Block::Empty;
            }
        }
        ToElement::ColorSwapper => {
            if layer != 2 {
                return;
            }
            if prev_block.is_connector() {
                *block = Block::ColorSwapper;
            } else {
                *block = Block::Empty;
            }
        }
        ToElement::Branch => {
            if *row > NUM_LINES - 3 {
                return;
            }
            let next_block = playground_prev_layer[*row + 1];
            if next_block.is_connector().not() {
                return;
            }
            let (prev_layer_block, prev_layer_next_blocks) =
                playground_prev_layer[*row..].split_first_mut().unwrap();
            if matches!(prev_layer_block, Block::BranchAbove | Block::BranchBelow) {
                return;
            }
            let next_next_block = &mut prev_layer_next_blocks[1];
            if matches!(next_next_block, Block::BranchAbove | Block::BranchBelow) {
                return;
            }
            cut_cable(prev_layer_block);
            cut_cable(next_next_block);

            *block = Block::BranchAbove;
            playground_layer[*row + 1] = Block::BranchMiddle;
            playground_layer[*row + 2] = Block::BranchBelow;
            *row += 2;
        }
        ToElement::Gate => {
            if *row > NUM_LINES - 3 {
                return;
            }

            let prev_layer_block = playground_prev_layer[*row];
            if prev_layer_block.is_connector().not() {
                return;
            }

            let next_next_block = playground_prev_layer[*row + 2];
            if next_next_block.is_connector().not() {
                return;
            }
            cut_cable(&mut playground_prev_layer[*row + 1]);

            *block = Block::GateAbove;
            playground_layer[*row + 1] = Block::GateMiddle;
            playground_layer[*row + 2] = Block::GateBelow;
            *row += 2;
        }
    }

    *row += 1;
}

#[derive(Debug, Clone, Copy)]
enum PlayTakeoverOnce {
    Continue,
    Finish,
    Return,
}
