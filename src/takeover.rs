/* Background-color of takeover-game */
pub const TO_BG_COLOR: usize = 63;

/* File containing the Takeover-blocks */
pub const TO_BLOCK_FILE: &str = "to_elem.png";

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
pub const CONNECTOR: usize = 0;
pub const NON_CONNECTOR: usize = 1;

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
pub type Playground = [[[i32; TO_COLORS]; NUM_LAYERS]; NUM_LINES];
