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
    convert::{TryFrom, TryInto},
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
pub const NUM_FILL_BLOCKS: usize = 3; // yellow, violett and black

/* Dimensions of a capsule */
pub const NUM_CAPS_BLOCKS: usize = 3; // yellow, violett and red (?what for)

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
    Gelb = 0,
    Violett,
    Remis,
}

/* Element - Names */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
enum ToElement {
    Kabel,
    Kabelende,
    Verstaerker,
    Farbtauscher,
    Verzweigung,
    Gatter,
}

/* Block-Names */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
enum ToBlock {
    Kabel,
    Kabelende,
    Verstaerker,
    Farbtauscher,
    VerzweigungO,
    VerzweigungM,
    VerzweigungU,
    GatterO,
    GatterM,
    GatterU,
    Leer,
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
                        && ToPlayground[opponent_color][0][row] != ToBlock::Kabelende as i32
                        && ActivationMap[opponent_color][0][row] == Condition::Inactive as i32 =>
                {
                    NumCapsules[ToOpponents::Enemy as usize] -= 1;
                    Takeover_Set_Capsule_Sound();
                    ToPlayground[opponent_color][0][row] = ToBlock::Verstaerker as i32;
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
                *playground = ToBlock::Kabel as i32;
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

    ActivationMap[ToColor::Gelb as usize][CONNECTION_LAYER]
        .iter()
        .zip(ActivationMap[ToColor::Violett as usize][CONNECTION_LAYER].iter())
        .zip(ToPlayground[ToColor::Gelb as usize][CONNECTION_LAYER - 1].iter())
        .zip(ToPlayground[ToColor::Violett as usize][CONNECTION_LAYER - 1].iter())
        .zip(DisplayColumn.iter_mut())
        .for_each(
            |(
                (((&gelb_activation, &violett_activation), &gelb_playground), &violett_playground),
                display,
            )| {
                if gelb_activation >= Condition::Active1 as i32
                    && violett_activation == Condition::Inactive as i32
                {
                    if gelb_playground == ToBlock::Farbtauscher as i32 {
                        *display = ToColor::Violett as i32;
                    } else {
                        *display = ToColor::Gelb as i32;
                    }
                } else if gelb_activation == Condition::Inactive as i32
                    && violett_activation >= Condition::Active1 as i32
                {
                    if violett_playground == ToBlock::Farbtauscher as i32 {
                        *display = ToColor::Gelb as i32;
                    } else {
                        *display = ToColor::Violett as i32;
                    }
                } else if gelb_activation >= Condition::Active1 as i32
                    && violett_activation >= Condition::Active1 as i32
                {
                    if gelb_playground == ToBlock::Farbtauscher as i32
                        && violett_playground != ToBlock::Farbtauscher as i32
                    {
                        *display = ToColor::Violett as i32;
                    } else if (gelb_playground != ToBlock::Farbtauscher as i32
                        && violett_playground == ToBlock::Farbtauscher as i32)
                        || FLICKER_COLOR == 0
                    {
                        *display = ToColor::Gelb as i32;
                    } else {
                        *display = ToColor::Violett as i32;
                    }
                }
            },
        );

    let mut gelb_counter = 0;
    let mut violett_counter = 0;
    for &color in DisplayColumn.iter() {
        if color == ToColor::Gelb as i32 {
            gelb_counter += 1;
        } else {
            violett_counter += 1;
        }
    }

    use std::cmp::Ordering;
    match violett_counter.cmp(&gelb_counter) {
        Ordering::Less => LeaderColor = ToColor::Gelb as i32,
        Ordering::Greater => LeaderColor = ToColor::Violett as i32,
        Ordering::Equal => LeaderColor = ToColor::Remis as i32,
    }
}
