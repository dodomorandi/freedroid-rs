use crate::{
    defs::{
        self, DisplayBannerFlags, Droid, MenuAction, Status, DROID_ROTATION_TIME, SHOW_WAIT, UPDATE,
    },
    enemy::class_of_druid,
    graphics::{clear_graph_mem, NE_SCREEN, TAKEOVER_BG_PIC},
    misc::my_random,
    structs::Point,
    vars::{DRUIDMAP, ME},
    view::fill_rect,
    Data, ALL_ENEMYS, DEATH_COUNT, INVINCIBLE_MODE, PRE_TAKE_ENERGY, REAL_SCORE,
};

use cstr::cstr;
use once_cell::sync::Lazy;
use sdl::{
    mouse::ll::{SDL_ShowCursor, SDL_DISABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{SDL_Color, SDL_Flip, SDL_SetClipRect, SDL_Surface, SDL_UpperBlit},
    Rect,
};
use std::{
    convert::{Infallible, TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::null_mut,
    sync::Mutex,
};

extern "C" {
    pub fn SDL_Delay(ms: u32);
}

static mut CAPSULE_CUR_ROW: [c_int; COLORS] = [0, 0];
static mut NUM_CAPSULES: [c_int; COLORS] = [0, 0];
static mut PLAYGROUND: [[[Block; NUM_LINES]; NUM_LAYERS]; COLORS] =
    [[[Block::Cable; NUM_LINES]; NUM_LAYERS]; COLORS];

static mut ACTIVATION_MAP: [[[Condition; NUM_LINES]; NUM_LAYERS]; COLORS] =
    [[[Condition::Inactive; NUM_LINES]; NUM_LAYERS]; COLORS];

static mut CAPSULES_COUNTDOWN: [[[Option<u8>; NUM_LINES]; NUM_LAYERS]; COLORS] =
    [[[None; NUM_LINES]; NUM_LAYERS]; COLORS];

static mut DISPLAY_COLUMN: [Color; NUM_LINES] = [
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
];

static mut LEADER_COLOR: Color = Color::Yellow;
static mut YOUR_COLOR: Color = Color::Yellow;
static mut OPPONENT_COLOR: Color = Color::Violet;
static mut DROID_NUM: c_int = 0;
static mut OPPONENT_TYPE: c_int = 0;

pub static TO_GAME_BLOCKS: Lazy<Mutex<[Rect; NUM_TO_BLOCKS]>> =
    Lazy::new(|| Mutex::new(array_init::array_init(|_| Rect::new(0, 0, 0, 0))));

pub static TO_GROUND_BLOCKS: Lazy<Mutex<[Rect; NUM_GROUND_BLOCKS]>> =
    Lazy::new(|| Mutex::new(array_init::array_init(|_| Rect::new(0, 0, 0, 0))));

pub static mut COLUMN_BLOCK: Rect = Rect {
    x: 0,
    y: 0,
    w: 0,
    h: 0,
};

pub static mut LEADER_BLOCK: Rect = Rect {
    x: 0,
    y: 0,
    w: 0,
    h: 0,
};

pub static mut LEFT_GROUND_START: Point = Point {
    x: 2 * 10,
    y: 2 * 15,
};

pub static mut RIGHT_GROUND_START: Point = Point {
    x: 2 * 255,
    y: 2 * 15,
};

pub static mut COLUMN_START: Point = Point {
    x: 2 * 136,
    y: 2 * 27,
};

pub static mut LEADER_BLOCK_START: Point = Point {
    x: 2 * 129,
    y: 2 * 8,
};

pub static mut LEADER_LED: Rect = Rect {
    x: 2 * 136,
    y: 2 * 11,
    w: 2 * 16,
    h: 2 * 19,
};

pub static mut FILL_BLOCK: Rect = Rect {
    x: 0,
    y: 0,
    w: 2 * 16,
    h: 2 * 7,
};

pub static mut ELEMENT_RECT: Rect = Rect {
    x: 0,
    y: 0,
    w: 2 * 32,
    h: 2 * 8,
};

pub static mut CAPSULE_RECT: Rect = Rect {
    x: 0,
    y: 0,
    w: 2 * 7,
    h: 2 * 8,
};

pub static mut GROUND_RECT: Rect = Rect {
    x: 0,
    y: 0,
    w: 2 * 23,
    h: 2 * 8,
};

pub static mut COLUMN_RECT: Rect = Rect {
    x: 0,
    y: 0,
    w: 2 * 30,
    h: 2 * 8,
};

pub static mut TO_BLOCKS: *mut SDL_Surface = null_mut(); /* the global surface containing all game-blocks */

/* the rectangles containing the blocks */
pub static mut FILL_BLOCKS: [Rect; NUM_FILL_BLOCKS] = [
    Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    },
    Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    },
    Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    },
];

pub static mut CAPSULE_BLOCKS: [Rect; NUM_CAPS_BLOCKS] = [
    Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    },
    Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    },
    Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    },
];

pub static mut LEFT_CAPSULE_STARTS: [Point; COLORS] = [
    Point { x: 4, y: 2 * 27 },
    Point {
        x: 2 * 255 + 2 * 30 - 10,
        y: 2 * 27,
    },
];

pub static mut CUR_CAPSULE_STARTS: [Point; COLORS] = [
    Point {
        x: 2 * 26,
        y: 2 * 19,
    },
    Point {
        x: 2 * 255,
        y: 2 * 19,
    },
];

pub static mut PLAYGROUND_STARTS: [Point; COLORS] = [
    Point {
        x: 2 * 33,
        y: 2 * 26,
    },
    Point {
        x: 2 * 159,
        y: 2 * 26,
    },
];
pub static mut DROID_STARTS: [Point; COLORS] =
    [Point { x: 2 * 40, y: -4 }, Point { x: 2 * 220, y: -4 }];

/* File containing the Takeover-blocks */
pub const TO_BLOCK_FILE_C: &CStr = cstr!("to_elem.png");

/* --------------- individual block dimensions --------------- */
const NUM_PHASES: usize =		5       /* number of color-phases for current "flow" */;
/* inclusive "inactive" phase */

/* Dimensions of the game-blocks */
const TO_BLOCKS_N: usize = 11; /* anzahl versch. Game- blocks */

const NUM_TO_BLOCKS: usize = 2 * NUM_PHASES * TO_BLOCKS_N; // total number of takover blocks
const TO_ELEMENTS: usize = 6;

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
        use Condition::*;
        match self {
            Inactive => false,
            Active1 | Active2 | Active3 | Active4 => true,
        }
    }

    const fn is_inactive(self) -> bool {
        !self.is_active()
    }

    fn next_active(self) -> Condition {
        use Condition::*;

        match self {
            Active1 => Active2,
            Active2 => Active3,
            Active3 => Active4,
            Active4 => Active1,
            Inactive => panic!("next_active called on inactive condition"),
        }
    }
}

impl From<Condition> for usize {
    fn from(condition: Condition) -> Self {
        use Condition::*;
        match condition {
            Inactive => 0,
            Active1 => 1,
            Active2 => 2,
            Active3 => 3,
            Active4 => 4,
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
                    use Color::*;
                    Ok(match value {
                        0 => Yellow,
                        1 => Violet,
                        2 => Draw,
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
        use Color::*;
        match color {
            Yellow => 0,
            Violet => 1,
            Draw => 2,
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
        use ToElement::*;
        Ok(match value {
            0 => Cable,
            1 => CableEnd,
            2 => Repeater,
            3 => ColorSwapper,
            4 => Branch,
            5 => Gate,
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
        use Block::*;
        match self {
            Cable => true,
            CableEnd => false,
            Repeater => true,
            ColorSwapper => true,
            BranchAbove => true,
            BranchMiddle => false,
            BranchBelow => true,
            GateAbove => false,
            GateMiddle => true,
            GateBelow => false,
            Empty => false,
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

/// Define all the SDL_Rects for the takeover-game
pub unsafe fn set_takeover_rects() -> c_int {
    /* Set the fill-blocks */
    FILL_BLOCKS
        .iter_mut()
        .zip((0..).step_by(usize::from(FILL_BLOCK.w) + 2))
        .for_each(|(rect, cur_x)| *rect = Rect::new(cur_x, 0, FILL_BLOCK.w, FILL_BLOCK.h));

    /* Set the capsule Blocks */
    let start_x =
        i16::try_from(FILL_BLOCKS.len()).unwrap() * (i16::try_from(FILL_BLOCK.w).unwrap() + 2);
    CAPSULE_BLOCKS
        .iter_mut()
        .zip((start_x..).step_by(usize::try_from(CAPSULE_RECT.w).unwrap() + 2))
        .for_each(|(rect, cur_x)| *rect = Rect::new(cur_x, 0, CAPSULE_RECT.w, CAPSULE_RECT.h - 2));

    /* get the game-blocks */
    TO_GAME_BLOCKS
        .lock()
        .unwrap()
        .iter_mut()
        .zip(
            ((FILL_BLOCK.h + 2)..)
                .step_by(usize::try_from(ELEMENT_RECT.h).unwrap() + 2)
                .flat_map(|cur_y| {
                    (0..)
                        .step_by(usize::try_from(ELEMENT_RECT.w).unwrap() + 2)
                        .take(TO_BLOCKS_N)
                        .map(move |cur_x| (cur_x, cur_y))
                }),
        )
        .for_each(|(rect, (cur_x, cur_y))| {
            *rect = Rect::new(
                cur_x,
                cur_y.try_into().unwrap(),
                ELEMENT_RECT.w,
                ELEMENT_RECT.h,
            )
        });
    let mut cur_y =
        (FILL_BLOCK.h + 2) + (ELEMENT_RECT.h + 2) * u16::try_from(NUM_PHASES).unwrap() * 2;

    /* Get the ground, column and leader blocks */
    TO_GROUND_BLOCKS
        .lock()
        .unwrap()
        .iter_mut()
        .zip((0..).step_by(usize::try_from(GROUND_RECT.w).unwrap() + 2))
        .for_each(|(rect, cur_x)| {
            *rect = Rect::new(
                cur_x,
                cur_y.try_into().unwrap(),
                GROUND_RECT.w,
                GROUND_RECT.h,
            )
        });
    cur_y += GROUND_RECT.h + 2;
    COLUMN_BLOCK = Rect::new(0, cur_y.try_into().unwrap(), COLUMN_RECT.w, COLUMN_RECT.h);
    LEADER_BLOCK = Rect::new(
        i16::try_from(COLUMN_RECT.w).unwrap() + 2,
        cur_y.try_into().unwrap(),
        LEADER_LED.w * 2 - 4,
        LEADER_LED.h,
    );
    defs::OK.into()
}

impl Data {
    unsafe fn enemy_movements(&self) {
        const ACTIONS: i32 = 3;
        const MOVE_PROBABILITY: i32 = 100;
        const TURN_PROBABILITY: i32 = 10;
        const SET_PROBABILITY: i32 = 80;

        static mut DIRECTION: i32 = 1; /* start with this direction */
        let opponent_color = OPPONENT_COLOR as usize;
        let mut row = CAPSULE_CUR_ROW[opponent_color] - 1;

        if NUM_CAPSULES[Opponents::Enemy as usize] == 0 {
            return;
        }

        let next_row = match my_random(ACTIONS) {
            0 => {
                /* Move along */
                if my_random(100) <= MOVE_PROBABILITY {
                    row += DIRECTION;
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

            1 => {
                /* Turn around */
                if my_random(100) <= TURN_PROBABILITY {
                    DIRECTION *= -1;
                }
                row + 1
            }

            2 => {
                /* Try to set  capsule */
                match usize::try_from(row) {
                    Ok(row)
                        if my_random(100) <= SET_PROBABILITY
                            && PLAYGROUND[opponent_color][0][row] != Block::CableEnd
                            && ACTIVATION_MAP[opponent_color][0][row] == Condition::Inactive =>
                    {
                        NUM_CAPSULES[Opponents::Enemy as usize] -= 1;
                        self.takeover_set_capsule_sound();
                        PLAYGROUND[opponent_color][0][row] = Block::Repeater;
                        ACTIVATION_MAP[opponent_color][0][row] = Condition::Active1;
                        CAPSULES_COUNTDOWN[opponent_color][0][row] = Some(CAPSULE_COUNTDOWN * 2);
                        0
                    }
                    _ => row + 1,
                }
            }
            _ => row + 1,
        };

        CAPSULE_CUR_ROW[opponent_color] = next_row;
    }
}

/// Animate the active cables: this is done by cycling over
/// the active phases ACTIVE1-ACTIVE3, which are represented by
/// different pictures in the playground
unsafe fn animate_currents() {
    ACTIVATION_MAP
        .iter_mut()
        .flat_map(|color_map| color_map.iter_mut())
        .flat_map(|layer_map| layer_map.iter_mut())
        .filter(|condition| condition.is_active())
        .for_each(|condition| *condition = condition.next_active());
}

unsafe fn is_active(color: c_int, row: c_int) -> c_int {
    const CONNECTION_LAYER: usize = 3; /* the connective Layer */
    let test_element = PLAYGROUND[usize::try_from(color).unwrap()][CONNECTION_LAYER - 1]
        [usize::try_from(row).unwrap()];

    if ACTIVATION_MAP[usize::try_from(color).unwrap()][CONNECTION_LAYER - 1]
        [usize::try_from(row).unwrap()]
    .is_active()
        && test_element.is_connector()
    {
        true.into()
    } else {
        false.into()
    }
}

/// does the countdown of the capsules and kills them if too old
unsafe fn process_capsules() {
    CAPSULES_COUNTDOWN
        .iter_mut()
        .flat_map(|color_countdown| color_countdown.iter_mut())
        .map(|countdown| &mut countdown[0])
        .zip(
            ACTIVATION_MAP
                .iter_mut()
                .flat_map(|color_activation| color_activation.iter_mut())
                .map(|activation| &mut activation[0]),
        )
        .zip(
            PLAYGROUND
                .iter_mut()
                .flat_map(|color_playground| color_playground.iter_mut())
                .map(|playground| &mut playground[0]),
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

unsafe fn process_display_column() {
    const CONNECTION_LAYER: usize = 3;
    static mut FLICKER_COLOR: i32 = 0;

    FLICKER_COLOR = !FLICKER_COLOR;

    ACTIVATION_MAP[Color::Yellow as usize][CONNECTION_LAYER]
        .iter()
        .zip(ACTIVATION_MAP[Color::Violet as usize][CONNECTION_LAYER].iter())
        .zip(PLAYGROUND[Color::Yellow as usize][CONNECTION_LAYER - 1].iter())
        .zip(PLAYGROUND[Color::Violet as usize][CONNECTION_LAYER - 1].iter())
        .zip(DISPLAY_COLUMN.iter_mut())
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
                        || FLICKER_COLOR == 0
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
    for &color in DISPLAY_COLUMN.iter() {
        if color == Color::Yellow {
            yellow_counter += 1;
        } else {
            violet_counter += 1;
        }
    }

    use std::cmp::Ordering;
    match violet_counter.cmp(&yellow_counter) {
        Ordering::Less => LEADER_COLOR = Color::Yellow,
        Ordering::Greater => LEADER_COLOR = Color::Violet,
        Ordering::Equal => LEADER_COLOR = Color::Draw,
    }
}

/// process the playground following its intrinsic logic
unsafe fn process_playground() {
    ACTIVATION_MAP
        .iter_mut()
        .zip(PLAYGROUND.iter())
        .enumerate()
        .for_each(|(color, (activation_color, playground_color))| {
            playground_color
                .iter()
                .enumerate()
                .skip(1)
                .for_each(|(layer, playground_layer)| {
                    let (activation_layer_last, activation_layer) =
                        activation_color.split_at_mut(layer);
                    let activation_layer_last = activation_layer_last.last().unwrap();
                    let activation_layer = &mut activation_layer[0];

                    playground_layer
                        .iter()
                        .enumerate()
                        .for_each(|(row, playground)| {
                            process_playground_row(
                                row,
                                playground,
                                activation_layer_last,
                                activation_layer,
                            )
                        });
                });

            activation_color
                .last_mut()
                .unwrap()
                .iter_mut()
                .enumerate()
                .for_each(|(row, activation)| {
                    if is_active(color.try_into().unwrap(), row.try_into().unwrap()) != 0 {
                        *activation = Condition::Active1;
                    } else {
                        *activation = Condition::Inactive;
                    }
                });
        });
}

#[inline]
fn process_playground_row(
    row: usize,
    playground: &Block,
    activation_layer_last: &[Condition],
    activation_layer: &mut [Condition],
) {
    let activation_last_layer = activation_layer_last[row];
    let (activation_last, activation_layer) = activation_layer.split_at_mut(row);
    let activation_last = activation_last.last().copied();
    let (activation, activation_layer) = activation_layer.split_first_mut().unwrap();
    let activation_next = activation_layer.first().copied();

    use Block::*;
    let turn_active = match playground {
        ColorSwapper | BranchMiddle | GateAbove | GateBelow | Cable => {
            activation_last_layer.is_active()
        }

        Repeater => activation_last_layer.is_active() || activation.is_active(),

        BranchAbove => activation_next
            .map(|condition| condition.is_active())
            .unwrap_or(false),

        BranchBelow => activation_last
            .map(|condition| condition.is_active())
            .unwrap_or(false),

        GateMiddle => {
            activation_last
                .map(|condition| condition.is_active())
                .unwrap_or(false)
                && activation_next
                    .map(|condition| condition.is_active())
                    .unwrap_or(false)
        }

        CableEnd | Empty => false,
    };

    if turn_active {
        if activation.is_inactive() {
            *activation = Condition::Active1;
        }
    } else {
        *activation = Condition::Inactive;
    }
}

/// generate a random Playground
unsafe fn invent_playground() {
    use std::ops::Not;

    const MAX_PROB: i32 = 100;
    const ELEMENTS_PROBABILITIES: [i32; TO_ELEMENTS] = [
        100, /* Cable */
        2,   /* CableEnd */
        5,   /* Repeater */
        5,   /* ColorSwapper: only on last layer */
        5,   /* Branch */
        5,   /* Gate */
    ];

    fn cut_cable(block: &mut Block) {
        if block.is_connector() {
            *block = Block::CableEnd;
        }
    }

    /* first clear the playground: we depend on this !! */
    clear_playground();

    PLAYGROUND.iter_mut().for_each(|playground_color| {
        for layer in 1..NUM_LAYERS {
            let (playground_prev_layers, playground_layer) = playground_color.split_at_mut(layer);
            let playground_prev_layer = playground_prev_layers.last_mut().unwrap();
            let playground_layer = &mut playground_layer[0];

            let mut row = 0;
            while row < NUM_LINES {
                let block = &mut playground_layer[row];
                if !matches!(block, Block::Cable) {
                    row += 1;
                    continue;
                }

                let new_element =
                    u8::try_from(my_random((TO_ELEMENTS - 1).try_into().unwrap())).unwrap();
                if my_random(MAX_PROB) > ELEMENTS_PROBABILITIES[usize::from(new_element)] {
                    continue;
                }

                let prev_block = playground_prev_layer[row];
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
                            continue;
                        }
                        if prev_block.is_connector() {
                            *block = Block::ColorSwapper;
                        } else {
                            *block = Block::Empty;
                        }
                    }
                    ToElement::Branch => {
                        if row > NUM_LINES - 3 {
                            continue;
                        }
                        let next_block = playground_prev_layer[row + 1];
                        if next_block.is_connector().not() {
                            continue;
                        }
                        let (prev_layer_block, prev_layer_next_blocks) =
                            playground_prev_layer[row..].split_first_mut().unwrap();
                        if matches!(prev_layer_block, Block::BranchAbove | Block::BranchBelow) {
                            continue;
                        }
                        let next_next_block = &mut prev_layer_next_blocks[1];
                        if matches!(next_next_block, Block::BranchAbove | Block::BranchBelow) {
                            continue;
                        }
                        cut_cable(prev_layer_block);
                        cut_cable(next_next_block);

                        *block = Block::BranchAbove;
                        playground_layer[row + 1] = Block::BranchMiddle;
                        playground_layer[row + 2] = Block::BranchBelow;
                        row += 2;
                    }
                    ToElement::Gate => {
                        if row > NUM_LINES - 3 {
                            continue;
                        }

                        let prev_layer_block = playground_prev_layer[row];
                        if prev_layer_block.is_connector().not() {
                            continue;
                        }

                        let next_next_block = playground_prev_layer[row + 2];
                        if next_next_block.is_connector().not() {
                            continue;
                        }
                        cut_cable(&mut playground_prev_layer[row + 1]);

                        *block = Block::GateAbove;
                        playground_layer[row + 1] = Block::GateMiddle;
                        playground_layer[row + 2] = Block::GateBelow;
                        row += 2;
                    }
                }

                row += 1;
            }
        }
    });
}

/// Clears Playground (and ACTIVATION_MAP) to default start-values
unsafe fn clear_playground() {
    ACTIVATION_MAP
        .iter_mut()
        .flatten()
        .flatten()
        .for_each(|activation| *activation = Condition::Inactive);

    PLAYGROUND
        .iter_mut()
        .flatten()
        .flatten()
        .for_each(|block| *block = Block::Cable);

    DISPLAY_COLUMN
        .iter_mut()
        .enumerate()
        .for_each(|(row, display_column)| *display_column = (row % 2).try_into().unwrap());
}

impl Data {
    /// prepares _and displays_ the current Playground
    ///
    /// NOTE: this function should only change the USERFENSTER part
    ///       so that we can do Infoline-setting before this
    unsafe fn show_playground(&mut self) {
        let your_color: usize = YOUR_COLOR.into();
        let opponent_color: usize = OPPONENT_COLOR.into();

        let xoffs = self.vars.classic_user_rect.x;
        let yoffs = self.vars.classic_user_rect.y;

        SDL_SetClipRect(NE_SCREEN, null_mut());

        SDL_UpperBlit(
            TAKEOVER_BG_PIC,
            &mut self.vars.user_rect,
            NE_SCREEN,
            &mut self.vars.user_rect,
        );

        self.put_influence(
            i32::from(xoffs) + DROID_STARTS[your_color].x,
            i32::from(yoffs) + DROID_STARTS[your_color].y,
        );

        if ALL_ENEMYS[usize::try_from(DROID_NUM).unwrap()].status != Status::Out as i32 {
            self.put_enemy(
                DROID_NUM,
                i32::from(xoffs) + DROID_STARTS[opponent_color].x,
                i32::from(yoffs) + DROID_STARTS[opponent_color].y,
            );
        }

        let mut dst = Rect::new(
            xoffs + i16::try_from(LEFT_GROUND_START.x).unwrap(),
            yoffs + i16::try_from(LEFT_GROUND_START.y).unwrap(),
            self.vars.user_rect.w,
            self.vars.user_rect.h,
        );

        let mut to_ground_blocks = TO_GROUND_BLOCKS.lock().unwrap();
        SDL_UpperBlit(
            TO_BLOCKS,
            &mut to_ground_blocks[GroundBlock::YellowAbove as usize],
            NE_SCREEN,
            &mut dst,
        );

        dst.y += i16::try_from(GROUND_RECT.h).unwrap();

        for _ in 0..12 {
            SDL_UpperBlit(
                TO_BLOCKS,
                &mut to_ground_blocks[GroundBlock::YellowMiddle as usize],
                NE_SCREEN,
                &mut dst,
            );

            dst.y += i16::try_from(GROUND_RECT.h).unwrap();
        }

        SDL_UpperBlit(
            TO_BLOCKS,
            &mut to_ground_blocks[GroundBlock::YellowBelow as usize],
            NE_SCREEN,
            &mut dst,
        );

        dst = Rect::new(
            xoffs + i16::try_from(LEADER_BLOCK_START.x).unwrap(),
            yoffs + i16::try_from(LEADER_BLOCK_START.y).unwrap(),
            0,
            0,
        );
        SDL_UpperBlit(TO_BLOCKS, &mut LEADER_BLOCK, NE_SCREEN, &mut dst);

        dst.y += i16::try_from(LEADER_LED.h).unwrap();
        for _ in 0..12 {
            SDL_UpperBlit(TO_BLOCKS, &mut COLUMN_BLOCK, NE_SCREEN, &mut dst);
            dst.y += i16::try_from(COLUMN_RECT.h).unwrap();
        }

        /* rechte Saeule */
        dst = Rect::new(
            xoffs + i16::try_from(RIGHT_GROUND_START.x).unwrap(),
            yoffs + i16::try_from(RIGHT_GROUND_START.y).unwrap(),
            0,
            0,
        );

        SDL_UpperBlit(
            TO_BLOCKS,
            &mut to_ground_blocks[GroundBlock::VioletAbove as usize],
            NE_SCREEN,
            &mut dst,
        );
        dst.y += i16::try_from(GROUND_RECT.h).unwrap();

        for _ in 0..12 {
            SDL_UpperBlit(
                TO_BLOCKS,
                &mut to_ground_blocks[GroundBlock::VioletMiddle as usize],
                NE_SCREEN,
                &mut dst,
            );
            dst.y += i16::try_from(GROUND_RECT.h).unwrap();
        }

        SDL_UpperBlit(
            TO_BLOCKS,
            &mut to_ground_blocks[GroundBlock::VioletBelow as usize],
            NE_SCREEN,
            &mut dst,
        );
        drop(to_ground_blocks);

        /* Fill the Leader-LED with its color */
        let leader_color = usize::try_from(LEADER_COLOR).unwrap();
        dst = Rect::new(xoffs + LEADER_LED.x, yoffs + LEADER_LED.y, 0, 0);
        SDL_UpperBlit(
            TO_BLOCKS,
            &mut FILL_BLOCKS[leader_color],
            NE_SCREEN,
            &mut dst,
        );
        dst.y += i16::try_from(FILL_BLOCK.h).unwrap();
        SDL_UpperBlit(
            TO_BLOCKS,
            &mut FILL_BLOCKS[leader_color],
            NE_SCREEN,
            &mut dst,
        );

        /* Fill the Display Column with its leds */
        DISPLAY_COLUMN
            .iter()
            .copied()
            .enumerate()
            .for_each(|(line, display_column)| {
                dst = Rect::new(
                    xoffs + i16::try_from(COLUMN_START.x).unwrap(),
                    yoffs
                        + i16::try_from(COLUMN_START.y).unwrap()
                        + i16::try_from(line).unwrap() * i16::try_from(COLUMN_RECT.h).unwrap(),
                    0,
                    0,
                );
                SDL_UpperBlit(
                    TO_BLOCKS,
                    &mut FILL_BLOCKS[usize::try_from(display_column).unwrap()],
                    NE_SCREEN,
                    &mut dst,
                );
            });

        /* Show the yellow playground */
        let mut to_game_blocks = TO_GAME_BLOCKS.lock().unwrap();
        PLAYGROUND[Color::Yellow as usize]
            .iter()
            .take(NUM_LAYERS - 1)
            .zip(
                ACTIVATION_MAP[Color::Yellow as usize]
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
                    dst = Rect::new(
                        xoffs
                            + i16::try_from(PLAYGROUND_STARTS[Color::Yellow as usize].x).unwrap()
                            + layer_index * i16::try_from(ELEMENT_RECT.w).unwrap(),
                        yoffs
                            + i16::try_from(PLAYGROUND_STARTS[Color::Yellow as usize].y).unwrap()
                            + line_index * i16::try_from(ELEMENT_RECT.h).unwrap(),
                        0,
                        0,
                    );

                    let block = playground_line + activation_line * TO_BLOCKS_N;
                    SDL_UpperBlit(TO_BLOCKS, &mut to_game_blocks[block], NE_SCREEN, &mut dst);
                },
            );

        /* Show the violet playground */
        PLAYGROUND[Color::Violet as usize]
            .iter()
            .take(NUM_LAYERS - 1)
            .zip(
                ACTIVATION_MAP[Color::Violet as usize]
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
                            usize::try_from(playground_line).unwrap(),
                            usize::try_from(activation_line).unwrap(),
                        )
                    })
            })
            .for_each(
                |(layer_index, line_index, playground_line, activation_line)| {
                    dst = Rect::new(
                        xoffs
                            + i16::try_from(PLAYGROUND_STARTS[Color::Violet as usize].x).unwrap()
                            + (i16::try_from(NUM_LAYERS).unwrap() - layer_index - 2)
                                * i16::try_from(ELEMENT_RECT.w).unwrap(),
                        yoffs
                            + i16::try_from(PLAYGROUND_STARTS[Color::Violet as usize].y).unwrap()
                            + line_index * i16::try_from(ELEMENT_RECT.h).unwrap(),
                        0,
                        0,
                    );
                    let block = playground_line + (NUM_PHASES + activation_line) * TO_BLOCKS_N;
                    SDL_UpperBlit(TO_BLOCKS, &mut to_game_blocks[block], NE_SCREEN, &mut dst);
                },
            );

        /* Show the capsules left for each player */
        NUM_CAPSULES
            .iter()
            .copied()
            .enumerate()
            .for_each(|(player, capsules)| {
                let color = if player == Opponents::You as usize {
                    your_color
                } else {
                    opponent_color
                };

                dst = Rect::new(
                    xoffs + i16::try_from(CUR_CAPSULE_STARTS[color].x).unwrap(),
                    yoffs
                        + i16::try_from(CUR_CAPSULE_STARTS[color].y).unwrap()
                        + i16::try_from(CAPSULE_CUR_ROW[color]).unwrap()
                            * i16::try_from(CAPSULE_RECT.h).unwrap(),
                    0,
                    0,
                );
                if capsules != 0 {
                    SDL_UpperBlit(TO_BLOCKS, &mut CAPSULE_BLOCKS[color], NE_SCREEN, &mut dst);
                }

                for capsule in 0..capsules.saturating_sub(1) {
                    dst = Rect::new(
                        xoffs + i16::try_from(LEFT_CAPSULE_STARTS[color].x).unwrap(),
                        yoffs
                            + i16::try_from(LEFT_CAPSULE_STARTS[color].y).unwrap()
                            + i16::try_from(capsule).unwrap()
                                * i16::try_from(CAPSULE_RECT.h).unwrap(),
                        0,
                        0,
                    );
                    SDL_UpperBlit(TO_BLOCKS, &mut CAPSULE_BLOCKS[color], NE_SCREEN, &mut dst);
                }
            });
    }

    /// the acutal Takeover game-playing is done here
    unsafe fn play_game(&mut self) {
        let mut countdown = 100;

        const COUNT_TICK_LEN: u32 = 100;
        const MOVE_TICK_LEN: u32 = 60;

        let mut prev_count_tick = SDL_GetTicks();
        let mut prev_move_tick = prev_count_tick;

        self.wait_for_all_keys_released();

        self.countdown_sound();
        let mut finish_takeover = false;
        let your_color = usize::try_from(YOUR_COLOR).unwrap();
        while !finish_takeover {
            let cur_time = SDL_GetTicks();

            let do_update_count = cur_time > prev_count_tick + COUNT_TICK_LEN;
            if do_update_count {
                /* time to count 1 down */
                prev_count_tick += COUNT_TICK_LEN; /* set for next countdown tick */
                countdown -= 1;
                let count_text = format!("Finish-{}\0", countdown);
                self.display_banner(
                    count_text.as_bytes().as_ptr() as *const c_char,
                    null_mut(),
                    0,
                );

                if countdown != 0 && countdown % 10 == 0 {
                    self.countdown_sound();
                }
                if countdown == 0 {
                    self.end_countdown_sound();
                    finish_takeover = true;
                }

                animate_currents(); /* do some animation on the active cables */
            }

            let do_update_move = cur_time > prev_move_tick + MOVE_TICK_LEN;
            if do_update_move {
                prev_move_tick += MOVE_TICK_LEN; /* set for next motion tick */

                let key_repeat_delay = if cfg!(target_os = "android") {
                    150 // better to avoid accidential key-repeats on touchscreen
                } else {
                    110 // PC default, allows for quick-repeat key hits
                };

                let action = self.get_menu_action(key_repeat_delay);
                /* allow for a WIN-key that give immedate victory */
                if self.key_is_pressed_r(b'w'.into()) && self.ctrl_pressed() && self.alt_pressed() {
                    LEADER_COLOR = YOUR_COLOR; /* simple as that */
                    return;
                }

                if action.intersects(MenuAction::UP | MenuAction::UP_WHEEL) {
                    CAPSULE_CUR_ROW[your_color] -= 1;
                    if CAPSULE_CUR_ROW[your_color] < 1 {
                        CAPSULE_CUR_ROW[your_color] = NUM_LINES.try_into().unwrap();
                    }
                }

                if action.intersects(MenuAction::DOWN | MenuAction::DOWN_WHEEL) {
                    CAPSULE_CUR_ROW[your_color] += 1;
                    if CAPSULE_CUR_ROW[your_color] > NUM_LINES.try_into().unwrap() {
                        CAPSULE_CUR_ROW[your_color] = 1;
                    }
                }

                if action.intersects(MenuAction::CLICK) {
                    if let Ok(row) = usize::try_from(CAPSULE_CUR_ROW[your_color] - 1) {
                        if NUM_CAPSULES[Opponents::You as usize] > 0
                            && PLAYGROUND[your_color][0][row] != Block::CableEnd
                            && ACTIVATION_MAP[your_color][0][row] == Condition::Inactive
                        {
                            NUM_CAPSULES[Opponents::You as usize] -= 1;
                            CAPSULE_CUR_ROW[your_color] = 0;
                            PLAYGROUND[your_color][0][row] = Block::Repeater;
                            ACTIVATION_MAP[your_color][0][row] = Condition::Active1;
                            CAPSULES_COUNTDOWN[your_color][0][row] = Some(CAPSULE_COUNTDOWN * 2);
                            self.takeover_set_capsule_sound();
                        }
                    }
                }

                self.enemy_movements();
                process_capsules(); /* count down the lifetime of the capsules */

                process_playground();
                process_playground();
                process_playground();
                process_playground(); /* this has to be done several times to be sure */

                process_display_column();
                self.show_playground();
            } // if do_update_move

            SDL_Flip(NE_SCREEN);
            SDL_Delay(1);
        } /* while !FinishTakeover */

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
                SDL_Delay(COUNT_TICK_LEN);
            }
            if self.any_key_just_pressed() != 0 {
                fast_forward = true;
            }
            prev_count_tick += COUNT_TICK_LEN;
            process_capsules(); /* count down the lifetime of the capsules */
            process_capsules(); /* do it twice this time to be faster */
            animate_currents();
            process_playground();
            process_playground();
            process_playground();
            process_playground(); /* this has to be done several times to be sure */
            process_display_column();
            self.show_playground();
            SDL_Delay(1);
            SDL_Flip(NE_SCREEN);
        } /* while (countdown) */

        self.wait_for_all_keys_released();
    }

    unsafe fn choose_color(&mut self) {
        let mut countdown = 100; /* duration in 1/10 seconds given for color choosing */

        const COUNT_TICK_LEN: u32 = 100; /* countdown in 1/10 second steps */

        let mut prev_count_tick = SDL_GetTicks();

        self.wait_for_all_keys_released();

        let mut color_chosen = false;
        while !color_chosen {
            let action = self.get_menu_action(110);
            if action.intersects(MenuAction::RIGHT | MenuAction::DOWN_WHEEL) {
                if YOUR_COLOR != Color::Violet {
                    self.move_menu_position_sound();
                }
                YOUR_COLOR = Color::Violet;
                OPPONENT_COLOR = Color::Yellow;
            }

            if action.intersects(MenuAction::LEFT | MenuAction::UP_WHEEL) {
                if YOUR_COLOR != Color::Yellow {
                    self.move_menu_position_sound();
                }
                YOUR_COLOR = Color::Yellow;
                OPPONENT_COLOR = Color::Violet;
            }

            if action.intersects(MenuAction::CLICK) {
                color_chosen = true;
            }

            /* wait for next countdown tick */
            if SDL_GetTicks() >= prev_count_tick + COUNT_TICK_LEN {
                prev_count_tick += COUNT_TICK_LEN; /* set for next tick */
                countdown -= 1; /* Count down */
                let count_text = format!("Color-{}\0", countdown);

                self.display_banner(count_text.as_ptr() as *const c_char, null_mut(), 0);
                self.show_playground();
            }

            if countdown == 0 {
                color_chosen = true;
            }

            SDL_Flip(NE_SCREEN);
            SDL_Delay(1); // don't hog CPU
        }
    }

    /// play takeover-game against a druid
    ///
    /// Returns true if the user won, false otherwise
    pub unsafe fn takeover(&mut self, enemynum: c_int) -> c_int {
        static mut REJECT_ENERGY: c_int = 0; /* your energy if you're rejected */

        /* Prevent distortion of framerate by the delay coming from
         * the time spend in the menu.
         */
        self.activate_conservative_frame_computation();

        // Takeover game always uses Classic User_Rect:
        let buf = self.vars.user_rect;
        self.vars.user_rect = self.vars.classic_user_rect;

        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        const BG_COLOR: SDL_Color = SDL_Color {
            r: 130,
            g: 130,
            b: 130,
            unused: 0,
        };
        fill_rect(self.vars.user_rect, BG_COLOR);

        ME.status = Status::Mobile as i32; /* the new status _after_ the takeover game */

        SDL_ShowCursor(SDL_DISABLE); // no mouse-cursor in takeover game!

        self.show_droid_info(ME.ty, -1, 0);
        self.show_droid_portrait(
            self.vars.cons_droid_rect,
            ME.ty,
            DROID_ROTATION_TIME,
            UPDATE,
        );

        self.wait_for_all_keys_released();
        while !self.fire_pressed_r() {
            self.show_droid_portrait(self.vars.cons_droid_rect, ME.ty, DROID_ROTATION_TIME, 0);
            SDL_Delay(1);
        }

        let enemy_index: usize = enemynum.try_into().unwrap();
        self.show_droid_info(ALL_ENEMYS[enemy_index].ty, -2, 0);
        self.show_droid_portrait(
            self.vars.cons_droid_rect,
            ALL_ENEMYS[enemy_index].ty,
            DROID_ROTATION_TIME,
            UPDATE,
        );
        self.wait_for_all_keys_released();
        while !self.fire_pressed_r() {
            self.show_droid_portrait(
                self.vars.cons_droid_rect,
                ALL_ENEMYS[enemy_index].ty,
                DROID_ROTATION_TIME,
                0,
            );
            SDL_Delay(1);
        }

        SDL_UpperBlit(TAKEOVER_BG_PIC, null_mut(), NE_SCREEN, null_mut());
        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        self.wait_for_all_keys_released();
        let mut finish_takeover = false;
        while !finish_takeover {
            /* Init Color-column and Capsule-Number for each opponenet and your color */
            DISPLAY_COLUMN
                .iter_mut()
                .enumerate()
                .for_each(|(row, column)| *column = (row % 2).try_into().unwrap());
            CAPSULES_COUNTDOWN
                .iter_mut()
                .flat_map(|color_countdown| color_countdown[0].iter_mut())
                .for_each(|x| *x = None);

            YOUR_COLOR = Color::Yellow;
            OPPONENT_COLOR = Color::Violet;

            CAPSULE_CUR_ROW[usize::from(Color::Yellow)] = 0;
            CAPSULE_CUR_ROW[usize::from(Color::Violet)] = 0;

            DROID_NUM = enemynum;
            OPPONENT_TYPE = ALL_ENEMYS[enemy_index].ty;
            NUM_CAPSULES[Opponents::You as usize] = 3 + class_of_druid(ME.ty);
            NUM_CAPSULES[Opponents::Enemy as usize] = 4 + class_of_druid(OPPONENT_TYPE);

            invent_playground();

            self.show_playground();
            SDL_Flip(NE_SCREEN);

            self.choose_color();
            self.wait_for_all_keys_released();

            self.play_game();
            self.wait_for_all_keys_released();

            let message;
            /* Ausgang beurteilen und returnen */
            if INVINCIBLE_MODE != 0 || LEADER_COLOR == YOUR_COLOR {
                self.takeover_game_won_sound();
                if ME.ty == Droid::Droid001 as c_int {
                    REJECT_ENERGY = ME.energy as c_int;
                    PRE_TAKE_ENERGY = ME.energy as c_int;
                }

                // We provide some security agains too high energy/health values gained
                // by very rapid successions of successful takeover attempts
                let droid_map = std::slice::from_raw_parts(DRUIDMAP, Droid::NumDroids as usize);
                if ME.energy > droid_map[Droid::Droid001 as usize].maxenergy {
                    ME.energy = droid_map[Droid::Droid001 as usize].maxenergy;
                }
                if ME.health > droid_map[Droid::Droid001 as usize].maxenergy {
                    ME.health = droid_map[Droid::Droid001 as usize].maxenergy;
                }

                // We allow to gain the current energy/full health that was still in the
                // other droid, since all previous damage must be due to fighting damage,
                // and this is exactly the sort of damage can usually be cured in refreshes.
                ME.energy += ALL_ENEMYS[enemy_index].energy;
                ME.health += droid_map[usize::try_from(OPPONENT_TYPE).unwrap()].maxenergy;

                ME.ty = ALL_ENEMYS[enemy_index].ty;

                REAL_SCORE += droid_map[usize::try_from(OPPONENT_TYPE).unwrap()].score as f32;

                DEATH_COUNT += (OPPONENT_TYPE * OPPONENT_TYPE) as f32; // quadratic "importance", max=529

                ALL_ENEMYS[enemy_index].status = Status::Out as c_int; // removed droid silently (no blast!)

                if LEADER_COLOR != YOUR_COLOR {
                    /* only won because of InvincibleMode */
                    message = cstr!("You cheat")
                } else {
                    /* won the proper way */
                    message = cstr!("Complete")
                };

                finish_takeover = true;
            } else if LEADER_COLOR == OPPONENT_COLOR {
                /* LEADER_COLOR == YOUR_COLOR */
                // you lost, but enemy is killed too --> blast it!
                ALL_ENEMYS[enemy_index].energy = -1.0; /* to be sure */

                self.takeover_game_lost_sound();
                if ME.ty != Droid::Droid001 as c_int {
                    message = cstr!("Rejected");
                    ME.ty = Droid::Droid001 as c_int;
                    ME.energy = REJECT_ENERGY as f32;
                } else {
                    message = cstr!("Burnt Out");
                    ME.energy = 0.;
                }
                finish_takeover = true;
            } else {
                /* LeadColor == OPPONENT_COLOR */

                self.takeover_game_deadlock_sound();
                message = cstr!("Deadlock");
            }

            self.display_banner(message.as_ptr(), null_mut(), 0);
            self.show_playground();
            SDL_Flip(NE_SCREEN);

            self.wait_for_all_keys_released();
            let now = SDL_GetTicks();
            while !self.fire_pressed_r() && SDL_GetTicks() - now < SHOW_WAIT {
                #[cfg(target_os = "android")]
                SDL_Flip(NE_SCREEN);

                SDL_Delay(1);
            }
        }

        // restore User_Rect
        self.vars.user_rect = buf;

        clear_graph_mem();

        (LEADER_COLOR == YOUR_COLOR).into()
    }
}
