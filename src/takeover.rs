use crate::{
    defs::{self, AltPressed, CtrlPressed, MenuAction, Status},
    global::{
        to_blocks, AllEnemys, CapsuleBlocks, Classic_User_Rect, CurCapsuleStart, DruidStart,
        FillBlocks, LeftCapsulesStart, PlaygroundStart, TO_CapsuleRect, TO_ColumnRect,
        TO_ColumnStart, TO_ElementRect, TO_FillBlock, TO_GroundRect, TO_LeaderBlockStart,
        TO_LeaderLed, TO_LeftGroundStart, TO_RightGroundStart, ToColumnBlock, ToGameBlocks,
        ToGroundBlocks, ToLeaderBlock, User_Rect,
    },
    graphics::{ne_screen, takeover_bg_pic},
    input::{any_key_just_pressed, wait_for_all_keys_released, KeyIsPressedR},
    menu::getMenuAction,
    misc::MyRandom,
    sound::{CountdownSound, EndCountdownSound, Takeover_Set_Capsule_Sound},
    view::{DisplayBanner, PutEnemy, PutInfluence},
};

use cstr::cstr;
use sdl::{
    sdl::ll::SDL_GetTicks,
    video::ll::{SDL_Flip, SDL_SetClipRect, SDL_UpperBlit},
    Rect,
};
use std::{
    convert::{Infallible, TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::null_mut,
};

extern "C" {
    static mut CapsuleCurRow: [c_int; TO_COLORS];
    static mut NumCapsules: [c_int; TO_COLORS];
    static mut ToPlayground: Playground;
    static mut ActivationMap: Playground;
    static mut CapsuleCountdown: Playground;
    static mut BlockClass: [c_int; TO_BLOCKS];
    static mut DisplayColumn: [c_int; NUM_LINES];
    static mut LeaderColor: c_int;
    static mut YourColor: c_int;
    static mut OpponentColor: c_int;
    static mut DroidNum: c_int;

    pub fn SDL_Delay(ms: u32);
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
    YellowAbove,
    YellowMiddle,
    YellowBelow,
    VioletAbove,
    VioletMiddle,
    VioletBelow,
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

/// Clears Playground (and ActivationMap) to default start-values
#[no_mangle]
pub unsafe extern "C" fn ClearPlayground() {
    ActivationMap
        .iter_mut()
        .flatten()
        .flatten()
        .for_each(|activation| *activation = Condition::Inactive as i32);

    ToPlayground
        .iter_mut()
        .flatten()
        .flatten()
        .for_each(|block| *block = ToBlock::Cable as i32);

    DisplayColumn
        .iter_mut()
        .enumerate()
        .for_each(|(row, display_column)| *display_column = i32::try_from(row).unwrap() % 2);
}

/// prepares _and displays_ the current Playground
///
/// NOTE: this function should only change the USERFENSTER part
///       so that we can do Infoline-setting before this
#[no_mangle]
pub unsafe extern "C" fn ShowPlayground() {
    let your_color = usize::try_from(YourColor).unwrap();
    let opponent_color = usize::try_from(OpponentColor).unwrap();

    let xoffs = Classic_User_Rect.x;
    let yoffs = Classic_User_Rect.y;

    SDL_SetClipRect(ne_screen, null_mut());

    SDL_UpperBlit(takeover_bg_pic, &mut User_Rect, ne_screen, &mut User_Rect);

    PutInfluence(
        i32::from(xoffs) + DruidStart[your_color].x,
        i32::from(yoffs) + DruidStart[your_color].y,
    );

    if AllEnemys[usize::try_from(DroidNum).unwrap()].status != Status::Out as i32 {
        PutEnemy(
            DroidNum,
            i32::from(xoffs) + DruidStart[opponent_color].x,
            i32::from(yoffs) + DruidStart[opponent_color].y,
        );
    }

    let mut dst = Rect::new(
        xoffs + i16::try_from(TO_LeftGroundStart.x).unwrap(),
        yoffs + i16::try_from(TO_LeftGroundStart.y).unwrap(),
        User_Rect.w,
        User_Rect.h,
    );

    SDL_UpperBlit(
        to_blocks,
        &mut ToGroundBlocks[GroundBlock::YellowAbove as usize],
        ne_screen,
        &mut dst,
    );

    dst.y += i16::try_from(TO_GroundRect.h).unwrap();

    for _ in 0..12 {
        SDL_UpperBlit(
            to_blocks,
            &mut ToGroundBlocks[GroundBlock::YellowMiddle as usize],
            ne_screen,
            &mut dst,
        );

        dst.y += i16::try_from(TO_GroundRect.h).unwrap();
    }

    SDL_UpperBlit(
        to_blocks,
        &mut ToGroundBlocks[GroundBlock::YellowBelow as usize],
        ne_screen,
        &mut dst,
    );

    dst = Rect::new(
        xoffs + i16::try_from(TO_LeaderBlockStart.x).unwrap(),
        yoffs + i16::try_from(TO_LeaderBlockStart.y).unwrap(),
        0,
        0,
    );
    SDL_UpperBlit(to_blocks, &mut ToLeaderBlock, ne_screen, &mut dst);

    dst.y += i16::try_from(TO_LeaderLed.h).unwrap();
    for _ in 0..12 {
        SDL_UpperBlit(to_blocks, &mut ToColumnBlock, ne_screen, &mut dst);
        dst.y += i16::try_from(TO_ColumnRect.h).unwrap();
    }

    /* rechte Saeule */
    dst = Rect::new(
        xoffs + i16::try_from(TO_RightGroundStart.x).unwrap(),
        yoffs + i16::try_from(TO_RightGroundStart.y).unwrap(),
        0,
        0,
    );

    SDL_UpperBlit(
        to_blocks,
        &mut ToGroundBlocks[GroundBlock::VioletAbove as usize],
        ne_screen,
        &mut dst,
    );
    dst.y += i16::try_from(TO_GroundRect.h).unwrap();

    for _ in 0..12 {
        SDL_UpperBlit(
            to_blocks,
            &mut ToGroundBlocks[GroundBlock::VioletMiddle as usize],
            ne_screen,
            &mut dst,
        );
        dst.y += i16::try_from(TO_GroundRect.h).unwrap();
    }

    SDL_UpperBlit(
        to_blocks,
        &mut ToGroundBlocks[GroundBlock::VioletBelow as usize],
        ne_screen,
        &mut dst,
    );

    /* Fill the Leader-LED with its color */
    let leader_color = usize::try_from(LeaderColor).unwrap();
    dst = Rect::new(xoffs + TO_LeaderLed.x, yoffs + TO_LeaderLed.y, 0, 0);
    SDL_UpperBlit(
        to_blocks,
        &mut FillBlocks[leader_color],
        ne_screen,
        &mut dst,
    );
    dst.y += i16::try_from(TO_FillBlock.h).unwrap();
    SDL_UpperBlit(
        to_blocks,
        &mut FillBlocks[leader_color],
        ne_screen,
        &mut dst,
    );

    /* Fill the Display Column with its leds */
    DisplayColumn
        .iter()
        .copied()
        .enumerate()
        .for_each(|(line, display_column)| {
            dst = Rect::new(
                xoffs + i16::try_from(TO_ColumnStart.x).unwrap(),
                yoffs
                    + i16::try_from(TO_ColumnStart.y).unwrap()
                    + i16::try_from(line).unwrap() * i16::try_from(TO_ColumnRect.h).unwrap(),
                0,
                0,
            );
            SDL_UpperBlit(
                to_blocks,
                &mut FillBlocks[usize::try_from(display_column).unwrap()],
                ne_screen,
                &mut dst,
            );
        });

    /* Show the yellow playground */
    ToPlayground[ToColor::Yellow as usize]
        .iter()
        .take(NUM_LAYERS - 1)
        .zip(
            ActivationMap[ToColor::Yellow as usize]
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
                        + i16::try_from(PlaygroundStart[ToColor::Yellow as usize].x).unwrap()
                        + layer_index * i16::try_from(TO_ElementRect.w).unwrap(),
                    yoffs
                        + i16::try_from(PlaygroundStart[ToColor::Yellow as usize].y).unwrap()
                        + line_index * i16::try_from(TO_ElementRect.h).unwrap(),
                    0,
                    0,
                );

                let block = playground_line + activation_line * TO_BLOCKS;
                SDL_UpperBlit(to_blocks, &mut ToGameBlocks[block], ne_screen, &mut dst);
            },
        );

    /* Show the violet playground */
    ToPlayground[ToColor::Violet as usize]
        .iter()
        .take(NUM_LAYERS - 1)
        .zip(
            ActivationMap[ToColor::Violet as usize]
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
                        + i16::try_from(PlaygroundStart[ToColor::Violet as usize].x).unwrap()
                        + (i16::try_from(NUM_LAYERS).unwrap() - layer_index - 2)
                            * i16::try_from(TO_ElementRect.w).unwrap(),
                    yoffs
                        + i16::try_from(PlaygroundStart[ToColor::Violet as usize].y).unwrap()
                        + line_index * i16::try_from(TO_ElementRect.h).unwrap(),
                    0,
                    0,
                );
                let block = playground_line + (NUM_PHASES + activation_line) * TO_BLOCKS;
                SDL_UpperBlit(to_blocks, &mut ToGameBlocks[block], ne_screen, &mut dst);
            },
        );

    /* Show the capsules left for each player */
    NumCapsules
        .iter()
        .copied()
        .enumerate()
        .for_each(|(player, capsules)| {
            let color = if player == ToOpponents::You as usize {
                your_color
            } else {
                opponent_color
            };

            dst = Rect::new(
                xoffs + i16::try_from(CurCapsuleStart[color].x).unwrap(),
                yoffs
                    + i16::try_from(CurCapsuleStart[color].y).unwrap()
                    + i16::try_from(CapsuleCurRow[color]).unwrap()
                        * i16::try_from(TO_CapsuleRect.h).unwrap(),
                0,
                0,
            );
            if capsules != 0 {
                SDL_UpperBlit(to_blocks, &mut CapsuleBlocks[color], ne_screen, &mut dst);
            }

            for capsule in 0..capsules.saturating_sub(1) {
                dst = Rect::new(
                    xoffs + i16::try_from(LeftCapsulesStart[color].x).unwrap(),
                    yoffs
                        + i16::try_from(LeftCapsulesStart[color].y).unwrap()
                        + i16::try_from(capsule).unwrap()
                            * i16::try_from(TO_CapsuleRect.h).unwrap(),
                    0,
                    0,
                );
                SDL_UpperBlit(to_blocks, &mut CapsuleBlocks[color], ne_screen, &mut dst);
            }
        });
}

/// the acutal Takeover game-playing is done here
#[no_mangle]
pub unsafe extern "C" fn PlayGame() {
    let mut countdown = 100;

    const COUNT_TICK_LEN: u32 = 100;
    const MOVE_TICK_LEN: u32 = 60;

    let mut prev_count_tick = SDL_GetTicks();
    let mut prev_move_tick = prev_count_tick;

    wait_for_all_keys_released();

    CountdownSound();
    let mut finish_takeover = false;
    let your_color = usize::try_from(YourColor).unwrap();
    while !finish_takeover {
        let cur_time = SDL_GetTicks();

        let do_update_count = cur_time > prev_count_tick + COUNT_TICK_LEN;
        if do_update_count {
            /* time to count 1 down */
            prev_count_tick += COUNT_TICK_LEN; /* set for next countdown tick */
            countdown -= 1;
            let count_text = format!("Finish-{}\0", countdown);
            DisplayBanner(
                count_text.as_bytes().as_ptr() as *const c_char,
                null_mut(),
                0,
            );

            if countdown != 0 && countdown % 10 == 0 {
                CountdownSound();
            }
            if countdown == 0 {
                EndCountdownSound();
                finish_takeover = true;
            }

            AnimateCurrents(); /* do some animation on the active cables */
        }

        let do_update_move = cur_time > prev_move_tick + MOVE_TICK_LEN;
        if do_update_move {
            prev_move_tick += MOVE_TICK_LEN; /* set for next motion tick */

            let key_repeat_delay = if cfg!(target_os = "android") {
                150 // better to avoid accidential key-repeats on touchscreen
            } else {
                110 // PC default, allows for quick-repeat key hits
            };

            let action = getMenuAction(key_repeat_delay);
            /* allow for a WIN-key that give immedate victory */
            if KeyIsPressedR(b'w'.into()) && CtrlPressed() && AltPressed() {
                LeaderColor = YourColor; /* simple as that */
                return;
            }

            if action.intersects(MenuAction::UP | MenuAction::UP_WHEEL) {
                CapsuleCurRow[your_color] -= 1;
                if CapsuleCurRow[your_color] < 1 {
                    CapsuleCurRow[your_color] = NUM_LINES.try_into().unwrap();
                }
            }

            if action.intersects(MenuAction::DOWN | MenuAction::DOWN_WHEEL) {
                CapsuleCurRow[your_color] += 1;
                if CapsuleCurRow[your_color] > NUM_LINES.try_into().unwrap() {
                    CapsuleCurRow[your_color] = 1;
                }
            }

            if action.intersects(MenuAction::CLICK) {
                if let Ok(row) = usize::try_from(CapsuleCurRow[your_color] - 1) {
                    if NumCapsules[ToOpponents::You as usize] > 0
                        && ToPlayground[your_color][0][row] != ToBlock::CableEnd as i32
                        && ActivationMap[your_color][0][row] == Condition::Inactive as i32
                    {
                        NumCapsules[ToOpponents::You as usize] -= 1;
                        CapsuleCurRow[your_color] = 0;
                        ToPlayground[your_color][0][row] = ToBlock::Repeater as i32;
                        ActivationMap[your_color][0][row] = Condition::Active1 as i32;
                        CapsuleCountdown[your_color][0][row] =
                            i32::try_from(CAPSULE_COUNTDOWN * 2).unwrap();
                        Takeover_Set_Capsule_Sound();
                    }
                }
            }

            EnemyMovements();
            ProcessCapsules(); /* count down the lifetime of the capsules */

            ProcessPlayground();
            ProcessPlayground();
            ProcessPlayground();
            ProcessPlayground(); /* this has to be done several times to be sure */

            ProcessDisplayColumn();
            ShowPlayground();
        } // if do_update_move

        SDL_Flip(ne_screen);
        SDL_Delay(1);
    } /* while !FinishTakeover */

    /* Schluss- Countdown */
    countdown = CAPSULE_COUNTDOWN;

    wait_for_all_keys_released();
    let mut fast_forward = false;
    loop {
        countdown -= 1;
        if countdown == 0 {
            break;
        }

        if !fast_forward {
            SDL_Delay(COUNT_TICK_LEN);
        }
        if any_key_just_pressed() != 0 {
            fast_forward = true;
        }
        prev_count_tick += COUNT_TICK_LEN;
        ProcessCapsules(); /* count down the lifetime of the capsules */
        ProcessCapsules(); /* do it twice this time to be faster */
        AnimateCurrents();
        ProcessPlayground();
        ProcessPlayground();
        ProcessPlayground();
        ProcessPlayground(); /* this has to be done several times to be sure */
        ProcessDisplayColumn();
        ShowPlayground();
        SDL_Delay(1);
        SDL_Flip(ne_screen);
    } /* while (countdown) */

    wait_for_all_keys_released();
}
