use crate::{
    defs,
    global::{
        CapsuleBlocks, FillBlocks, TO_CapsuleRect, TO_ColumnRect, TO_ElementRect, TO_FillBlock,
        TO_GroundRect, TO_LeaderLed, ToColumnBlock, ToGameBlocks, ToGroundBlocks, ToLeaderBlock,
    },
    misc::MyRandom,
    sound::Takeover_Set_Capsule_Sound,
};

use cstr::cstr;
use sdl::Rect;
use std::{
    convert::{Infallible, TryFrom, TryInto},
    ffi::CStr,
    os::raw::c_int,
};

extern "C" {
    static mut CapsuleCurRow: [c_int; TO_COLORS];
    static mut OpponentColor: c_int;
    static mut NumCapsules: [c_int; TO_COLORS];
    static mut ToPlayground: Playground;
    static mut ActivationMap: Playground;
    static mut CapsuleCountdown: Playground;
    static mut BlockClass: [c_int; TO_BLOCKS];
    static mut DisplayColumn: [c_int; NUM_LINES];
    static mut LeaderColor: c_int;

    fn ClearPlayground();
}

/* Background-color of takeover-game */
pub const TO_BG_COLOR: usize = 63;

/* File containing the Takeover-blocks */
pub const TO_BLOCK_FILE: &str = "to_elem.png";
pub const TO_BLOCK_FILE_C: &CStr = cstr!("to_elem.png");

/* --------------- individual block dimensions --------------- */
pub const NUM_PHASES: usize =		5       /* number of color-phases for current "flow" */;
/* inclusive "inactive" phase */

/* Dimensions of the game-blocks */
pub const TO_BLOCKS: usize = 11; /* anzahl versch. Game- blocks */

pub const NUM_TO_BLOCKS: usize = 2 * NUM_PHASES * TO_BLOCKS; // total number of takover blocks
pub const TO_ELEMENTS: usize = 6;

/* Dimensions of the fill-blocks (in led-column */
pub const NUM_FILL_BLOCKS: usize = 3; // yellow, violet and black

/* Dimensions of a capsule */
pub const NUM_CAPS_BLOCKS: usize = 3; // yellow, violet and red (?what for)

/* Dimensions of ground-, column- und leader blocks */
pub const NUM_GROUND_BLOCKS: usize = 6;

/* --------------- Timing parameters --------------- */
pub const COLOR_COUNTDOWN: usize = 100; /* Zeit zum Farbe auswaehlen */
pub const GAME_COUNTDOWN: usize = 100; /* Zeit fuer das Spiel */
pub const CAPSULE_COUNTDOWN: usize = 40; /* 1/10 sec. Lebensdauer einer Kapsel */

pub const WAIT_MOVEMENT: usize = 0; /* 1/18 sekunden Bewegungsgeschw. */
pub const WAIT_COLOR_ROTATION: usize = 2; /* 1/18 sekunden aktive-Kabel */
pub const WAIT_AFTER_GAME: usize = 2 * 18; /* Wait after a deadlock */

pub const TO_TICK_LENGTH: usize = 40; /* Time in ms between ticks */

/* --------------- Playground layout --------------- */

pub const MAX_CAPSULES: usize = 13; /* a 999 has 13 !!! */

/* there are two classes of blocks: connectors and non-connectors */
pub const CONNECTOR: i32 = 0;
pub const NON_CONNECTOR: i32 = 1;

pub const NUM_LAYERS: usize = 4; /* dimension of the playground */
pub const NUM_LINES: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GroundBlock {
    GelbOben,
    GelbMitte,
    GelbUnten,
    ViolettOben,
    ViolettMitte,
    ViolettUnten,
}

/* Konditions in Connection-layer */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Condition {
    Inactive = 0,
    Active1,
    Active2,
    Active3,
    Active4,
}

/* Names for you and "him" */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ToOpponents {
    You,
    Enemy,
}

/* Color-names */
pub const TO_COLORS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ToColor {
    Yellow = 0,
    Violet,
    Draw,
}

/* Element - Names */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
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

/* Block-Names */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
enum ToBlock {
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

impl TryFrom<i32> for ToBlock {
    type Error = Infallible;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use ToBlock::*;
        Ok(match value {
            0 => Cable,
            1 => CableEnd,
            2 => Repeater,
            3 => ColorSwapper,
            4 => BranchAbove,
            5 => BranchMiddle,
            6 => BranchBelow,
            7 => GateAbove,
            8 => GateMiddle,
            9 => GateBelow,
            10 => Empty,
            _ => panic!("invalid raw ToBlock value"),
        })
    }
}

/* the playground type */
pub type Playground = [[[i32; NUM_LINES]; NUM_LAYERS]; TO_COLORS];

/// Define all the SDL_Rects for the takeover-game
#[no_mangle]
pub unsafe extern "C" fn set_takeover_rects() -> c_int {
    /* Set the fill-blocks */
    FillBlocks
        .iter_mut()
        .zip((0..).step_by(usize::from(TO_FillBlock.w) + 2))
        .for_each(|(rect, cur_x)| *rect = Rect::new(cur_x, 0, TO_FillBlock.w, TO_FillBlock.h));

    /* Set the capsule Blocks */
    let start_x =
        i16::try_from(FillBlocks.len()).unwrap() * (i16::try_from(TO_FillBlock.w).unwrap() + 2);
    CapsuleBlocks
        .iter_mut()
        .zip((start_x..).step_by(usize::try_from(TO_CapsuleRect.w).unwrap() + 2))
        .for_each(|(rect, cur_x)| {
            *rect = Rect::new(cur_x, 0, TO_CapsuleRect.w, TO_CapsuleRect.h - 2)
        });

    /* get the game-blocks */
    ToGameBlocks
        .iter_mut()
        .zip(
            ((TO_FillBlock.h + 2)..)
                .step_by(usize::try_from(TO_ElementRect.h).unwrap() + 2)
                .flat_map(|cur_y| {
                    (0..)
                        .step_by(usize::try_from(TO_ElementRect.w).unwrap() + 2)
                        .take(TO_BLOCKS)
                        .map(move |cur_x| (cur_x, cur_y))
                }),
        )
        .for_each(|(rect, (cur_x, cur_y))| {
            *rect = Rect::new(
                cur_x,
                cur_y.try_into().unwrap(),
                TO_ElementRect.w,
                TO_ElementRect.h,
            )
        });
    let mut cur_y =
        (TO_FillBlock.h + 2) + (TO_ElementRect.h + 2) * u16::try_from(NUM_PHASES).unwrap() * 2;

    /* Get the ground, column and leader blocks */
    ToGroundBlocks
        .iter_mut()
        .zip((0..).step_by(usize::try_from(TO_GroundRect.w).unwrap() + 2))
        .for_each(|(rect, cur_x)| {
            *rect = Rect::new(
                cur_x,
                cur_y.try_into().unwrap(),
                TO_GroundRect.w,
                TO_GroundRect.h,
            )
        });
    cur_y += TO_GroundRect.h + 2;
    ToColumnBlock = Rect::new(
        0,
        cur_y.try_into().unwrap(),
        TO_ColumnRect.w,
        TO_ColumnRect.h,
    );
    ToLeaderBlock = Rect::new(
        i16::try_from(TO_ColumnRect.w).unwrap() + 2,
        cur_y.try_into().unwrap(),
        TO_LeaderLed.w * 2 - 4,
        TO_LeaderLed.h,
    );
    defs::OK.into()
}

#[no_mangle]
pub unsafe extern "C" fn EnemyMovements() {
    const ACTIONS: i32 = 3;
    const MOVE_PROBABILITY: i32 = 100;
    const TURN_PROBABILITY: i32 = 10;
    const SET_PROBABILITY: i32 = 80;

    static mut DIRECTION: i32 = 1; /* start with this direction */
    let opponent_color = usize::try_from(OpponentColor).unwrap();
    let mut row = CapsuleCurRow[opponent_color] - 1;

    if NumCapsules[ToOpponents::Enemy as usize] == 0 {
        return;
    }

    let next_row = match MyRandom(ACTIONS) {
        0 => {
            /* Move along */
            if MyRandom(100) <= MOVE_PROBABILITY {
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
            if MyRandom(100) <= TURN_PROBABILITY {
                DIRECTION *= -1;
            }
            row + 1
        }

        2 => {
            /* Try to set  capsule */
            match usize::try_from(row) {
                Ok(row)
                    if MyRandom(100) <= SET_PROBABILITY
                        && ToPlayground[opponent_color][0][row] != ToBlock::CableEnd as i32
                        && ActivationMap[opponent_color][0][row] == Condition::Inactive as i32 =>
                {
                    NumCapsules[ToOpponents::Enemy as usize] -= 1;
                    Takeover_Set_Capsule_Sound();
                    ToPlayground[opponent_color][0][row] = ToBlock::Repeater as i32;
                    ActivationMap[opponent_color][0][row] = Condition::Active1 as i32;
                    CapsuleCountdown[opponent_color][0][row] =
                        i32::try_from(CAPSULE_COUNTDOWN).unwrap() * 2;
                    0
                }
                _ => row + 1,
            }
        }
        _ => row + 1,
    };

    CapsuleCurRow[opponent_color] = next_row;
}

/// Animate the active cables: this is done by cycling over
/// the active phases ACTIVE1-ACTIVE3, which are represented by
/// different pictures in the playground
#[no_mangle]
pub unsafe extern "C" fn AnimateCurrents() {
    ActivationMap
        .iter_mut()
        .flat_map(|color_map| color_map.iter_mut())
        .flat_map(|layer_map| layer_map.iter_mut())
        .filter(|condition| **condition >= Condition::Active1 as i32)
        .for_each(|condition| {
            *condition += 1;
            if *condition == NUM_PHASES.try_into().unwrap() {
                *condition = Condition::Active1 as i32;
            }
        });
}

#[no_mangle]
pub unsafe extern "C" fn IsActive(color: c_int, row: c_int) -> c_int {
    const CONNECTION_LAYER: usize = 3; /* the connective Layer */
    let test_element = ToPlayground[usize::try_from(color).unwrap()][CONNECTION_LAYER - 1]
        [usize::try_from(row).unwrap()];

    if ActivationMap[usize::try_from(color).unwrap()][CONNECTION_LAYER - 1]
        [usize::try_from(row).unwrap()]
        >= Condition::Active1 as i32
        && BlockClass[usize::try_from(test_element).unwrap()] == CONNECTOR
    {
        true.into()
    } else {
        false.into()
    }
}

/// does the countdown of the capsules and kills them if too old
#[no_mangle]
pub unsafe extern "C" fn ProcessCapsules() {
    CapsuleCountdown
        .iter_mut()
        .flat_map(|color_countdown| color_countdown.iter_mut())
        .map(|countdown| &mut countdown[0])
        .zip(
            ActivationMap
                .iter_mut()
                .flat_map(|color_activation| color_activation.iter_mut())
                .map(|activation| &mut activation[0]),
        )
        .zip(
            ToPlayground
                .iter_mut()
                .flat_map(|color_playground| color_playground.iter_mut())
                .map(|playground| &mut playground[0]),
        )
        .for_each(|((countdown, activation), playground)| {
            if *countdown > 0 {
                *countdown -= 1;
            }

            if *countdown == 0 {
                *countdown = -1;
                *activation = Condition::Inactive as i32;
                *playground = ToBlock::Cable as i32;
            }
        });
}

/// ProcessDisplayColumn(): setzt die Korrekten Werte in der Display-
/// Saeule. Blinkende LEDs werden ebenfalls hier realisiert
#[no_mangle]
pub unsafe extern "C" fn ProcessDisplayColumn() {
    const CONNECTION_LAYER: usize = 3;
    static mut FLICKER_COLOR: i32 = 0;

    FLICKER_COLOR = !FLICKER_COLOR;

    ActivationMap[ToColor::Yellow as usize][CONNECTION_LAYER]
        .iter()
        .zip(ActivationMap[ToColor::Violet as usize][CONNECTION_LAYER].iter())
        .zip(ToPlayground[ToColor::Yellow as usize][CONNECTION_LAYER - 1].iter())
        .zip(ToPlayground[ToColor::Violet as usize][CONNECTION_LAYER - 1].iter())
        .zip(DisplayColumn.iter_mut())
        .for_each(
            |(
                (
                    ((&yellow_activation, &violet_activation), &yellow_playground),
                    &violet_playground,
                ),
                display,
            )| {
                if yellow_activation >= Condition::Active1 as i32
                    && violet_activation == Condition::Inactive as i32
                {
                    if yellow_playground == ToBlock::ColorSwapper as i32 {
                        *display = ToColor::Violet as i32;
                    } else {
                        *display = ToColor::Yellow as i32;
                    }
                } else if yellow_activation == Condition::Inactive as i32
                    && violet_activation >= Condition::Active1 as i32
                {
                    if violet_playground == ToBlock::ColorSwapper as i32 {
                        *display = ToColor::Yellow as i32;
                    } else {
                        *display = ToColor::Violet as i32;
                    }
                } else if yellow_activation >= Condition::Active1 as i32
                    && violet_activation >= Condition::Active1 as i32
                {
                    if yellow_playground == ToBlock::ColorSwapper as i32
                        && violet_playground != ToBlock::ColorSwapper as i32
                    {
                        *display = ToColor::Violet as i32;
                    } else if (yellow_playground != ToBlock::ColorSwapper as i32
                        && violet_playground == ToBlock::ColorSwapper as i32)
                        || FLICKER_COLOR == 0
                    {
                        *display = ToColor::Yellow as i32;
                    } else {
                        *display = ToColor::Violet as i32;
                    }
                }
            },
        );

    let mut yellow_counter = 0;
    let mut violet_counter = 0;
    for &color in DisplayColumn.iter() {
        if color == ToColor::Yellow as i32 {
            yellow_counter += 1;
        } else {
            violet_counter += 1;
        }
    }

    use std::cmp::Ordering;
    match violet_counter.cmp(&yellow_counter) {
        Ordering::Less => LeaderColor = ToColor::Yellow as i32,
        Ordering::Greater => LeaderColor = ToColor::Violet as i32,
        Ordering::Equal => LeaderColor = ToColor::Draw as i32,
    }
}

/// process the playground following its intrinsic logic
#[no_mangle]
pub unsafe extern "C" fn ProcessPlayground() {
    ActivationMap
        .iter_mut()
        .zip(ToPlayground.iter())
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
                    if IsActive(color.try_into().unwrap(), row.try_into().unwrap()) != 0 {
                        *activation = Condition::Active1 as i32;
                    } else {
                        *activation = Condition::Inactive as i32;
                    }
                });
        });
}

#[inline]
fn process_playground_row(
    row: usize,
    playground: &i32,
    activation_layer_last: &[i32],
    activation_layer: &mut [i32],
) {
    let activation_last_layer = activation_layer_last[row];
    let (activation_last, activation_layer) = activation_layer.split_at_mut(row);
    let activation_last = activation_last.last().copied();
    let (activation, activation_layer) = activation_layer.split_first_mut().unwrap();
    let activation_next = activation_layer.first().copied();

    let playground = ToBlock::try_from(*playground).unwrap();
    use ToBlock::*;
    let turn_active = match playground {
        ColorSwapper | BranchMiddle | GateAbove | GateBelow | Cable => {
            activation_last_layer >= Condition::Active1 as i32
        }

        Repeater => {
            activation_last_layer >= Condition::Active1 as i32
                || *activation >= Condition::Active1 as i32
        }

        BranchAbove => activation_next
            .map(|value| value >= Condition::Active1 as i32)
            .unwrap_or(false),

        BranchBelow => activation_last
            .map(|value| value >= Condition::Active1 as i32)
            .unwrap_or(false),

        GateMiddle => {
            activation_last
                .map(|value| value >= Condition::Active1 as i32)
                .unwrap_or(false)
                && activation_next
                    .map(|value| value >= Condition::Active1 as i32)
                    .unwrap_or(false)
        }

        CableEnd | Empty => false,
    };

    if turn_active {
        if *activation == Condition::Inactive as i32 {
            *activation = Condition::Active1 as i32;
        }
    } else {
        *activation = Condition::Inactive as i32;
    }
}

/// generate a random Playground
#[no_mangle]
pub unsafe extern "C" fn InventPlayground() {
    const MAX_PROB: i32 = 100;
    const ELEMENTS_PROBABILITIES: [i32; TO_ELEMENTS] = [
        100, /* Cable */
        2,   /* CableEnd */
        5,   /* Repeater */
        5,   /* ColorSwapper: only on last layer */
        5,   /* Branch */
        5,   /* Gate */
    ];

    const BLOCK_IS_CONNECTOR: [bool; TO_BLOCKS] = [
        true,  /* Cable */
        false, /* CableEnd */
        true,  /* Repeater */
        true,  /* ColorSwapper */
        true,  /* BranchAbove */
        false, /* BranchMiddle */
        true,  /* BranchBelow */
        false, /* BranchAbove */
        true,  /* BranchMiddle */
        false, /* BranchBelow */
        false, /* Empty */
    ];

    fn cut_cable(block: &mut i32) {
        if BLOCK_IS_CONNECTOR[usize::try_from(*block).unwrap()] {
            *block = ToBlock::CableEnd as i32;
        }
    }

    /* first clear the playground: we depend on this !! */
    ClearPlayground();

    ToPlayground.iter_mut().for_each(|playground_color| {
        for layer in 1..NUM_LAYERS {
            let (playground_prev_layers, playground_layer) = playground_color.split_at_mut(layer);
            let playground_prev_layer = playground_prev_layers.last_mut().unwrap();
            let playground_layer = &mut playground_layer[0];

            let mut row = 0;
            while row < NUM_LINES {
                let block = &mut playground_layer[row];
                if !matches!((*block).try_into().unwrap(), ToBlock::Cable) {
                    row += 1;
                    continue;
                }

                let new_element =
                    u8::try_from(MyRandom((TO_ELEMENTS - 1).try_into().unwrap())).unwrap();
                if MyRandom(MAX_PROB) > ELEMENTS_PROBABILITIES[usize::from(new_element)] {
                    continue;
                }

                let prev_block = usize::try_from(playground_prev_layer[row]).unwrap();
                match ToElement::try_from(new_element).unwrap() {
                    ToElement::Cable => {
                        if !BLOCK_IS_CONNECTOR[prev_block] {
                            *block = ToBlock::Empty as i32;
                        }
                    }
                    ToElement::CableEnd => {
                        if BLOCK_IS_CONNECTOR[prev_block] {
                            *block = ToBlock::CableEnd as i32;
                        } else {
                            *block = ToBlock::Empty as i32;
                        }
                    }
                    ToElement::Repeater => {
                        if BLOCK_IS_CONNECTOR[prev_block] {
                            *block = ToBlock::Repeater as i32;
                        } else {
                            *block = ToBlock::Empty as i32;
                        }
                    }
                    ToElement::ColorSwapper => {
                        if layer != 2 {
                            continue;
                        }
                        if BLOCK_IS_CONNECTOR[prev_block] {
                            *block = ToBlock::ColorSwapper as i32;
                        } else {
                            *block = ToBlock::Empty as i32;
                        }
                    }
                    ToElement::Branch => {
                        if row > NUM_LINES - 3 {
                            continue;
                        }
                        let next_block = playground_prev_layer[row + 1];
                        if !BLOCK_IS_CONNECTOR[usize::try_from(next_block).unwrap()] {
                            continue;
                        }
                        let (prev_layer_block, prev_layer_next_blocks) =
                            playground_prev_layer[row..].split_first_mut().unwrap();
                        if matches!(
                            ToBlock::try_from(*prev_layer_block).unwrap(),
                            ToBlock::BranchAbove | ToBlock::BranchBelow
                        ) {
                            continue;
                        }
                        let next_next_block = &mut prev_layer_next_blocks[1];
                        if matches!(
                            ToBlock::try_from(*next_next_block).unwrap(),
                            ToBlock::BranchAbove | ToBlock::BranchBelow
                        ) {
                            continue;
                        }
                        cut_cable(prev_layer_block);
                        cut_cable(next_next_block);

                        *block = ToBlock::BranchAbove as i32;
                        playground_layer[row + 1] = ToBlock::BranchMiddle as i32;
                        playground_layer[row + 2] = ToBlock::BranchBelow as i32;
                        row += 2;
                    }
                    ToElement::Gate => {
                        if row > NUM_LINES - 3 {
                            continue;
                        }

                        let prev_layer_block = usize::try_from(playground_prev_layer[row]).unwrap();
                        if !BLOCK_IS_CONNECTOR[prev_layer_block] {
                            continue;
                        }

                        let next_next_block =
                            usize::try_from(playground_prev_layer[row + 2]).unwrap();
                        if !BLOCK_IS_CONNECTOR[next_next_block] {
                            continue;
                        }
                        cut_cable(&mut playground_prev_layer[row + 1]);

                        *block = ToBlock::GateAbove as i32;
                        playground_layer[row + 1] = ToBlock::GateMiddle as i32;
                        playground_layer[row + 2] = ToBlock::GateBelow as i32;
                        row += 2;
                    }
                }

                row += 1;
            }
        }
    });
}
