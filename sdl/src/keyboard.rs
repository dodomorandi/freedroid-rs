use bitflags::bitflags;
use sdl_sys::{
    SDLKey_SDLK_0, SDLKey_SDLK_1, SDLKey_SDLK_2, SDLKey_SDLK_3, SDLKey_SDLK_4, SDLKey_SDLK_5,
    SDLKey_SDLK_6, SDLKey_SDLK_7, SDLKey_SDLK_8, SDLKey_SDLK_9, SDLKey_SDLK_AMPERSAND,
    SDLKey_SDLK_ASTERISK, SDLKey_SDLK_AT, SDLKey_SDLK_BACKQUOTE, SDLKey_SDLK_BACKSLASH,
    SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_BREAK, SDLKey_SDLK_CAPSLOCK, SDLKey_SDLK_CARET,
    SDLKey_SDLK_CLEAR, SDLKey_SDLK_COLON, SDLKey_SDLK_COMMA, SDLKey_SDLK_COMPOSE,
    SDLKey_SDLK_DELETE, SDLKey_SDLK_DOLLAR, SDLKey_SDLK_DOWN, SDLKey_SDLK_END, SDLKey_SDLK_EQUALS,
    SDLKey_SDLK_ESCAPE, SDLKey_SDLK_EURO, SDLKey_SDLK_EXCLAIM, SDLKey_SDLK_F1, SDLKey_SDLK_F10,
    SDLKey_SDLK_F11, SDLKey_SDLK_F12, SDLKey_SDLK_F13, SDLKey_SDLK_F14, SDLKey_SDLK_F15,
    SDLKey_SDLK_F2, SDLKey_SDLK_F3, SDLKey_SDLK_F4, SDLKey_SDLK_F5, SDLKey_SDLK_F6, SDLKey_SDLK_F7,
    SDLKey_SDLK_F8, SDLKey_SDLK_F9, SDLKey_SDLK_FIRST, SDLKey_SDLK_GREATER, SDLKey_SDLK_HASH,
    SDLKey_SDLK_HELP, SDLKey_SDLK_HOME, SDLKey_SDLK_INSERT, SDLKey_SDLK_KP0, SDLKey_SDLK_KP1,
    SDLKey_SDLK_KP2, SDLKey_SDLK_KP3, SDLKey_SDLK_KP4, SDLKey_SDLK_KP5, SDLKey_SDLK_KP6,
    SDLKey_SDLK_KP7, SDLKey_SDLK_KP8, SDLKey_SDLK_KP9, SDLKey_SDLK_KP_DIVIDE, SDLKey_SDLK_KP_ENTER,
    SDLKey_SDLK_KP_EQUALS, SDLKey_SDLK_KP_MINUS, SDLKey_SDLK_KP_MULTIPLY, SDLKey_SDLK_KP_PERIOD,
    SDLKey_SDLK_KP_PLUS, SDLKey_SDLK_LALT, SDLKey_SDLK_LCTRL, SDLKey_SDLK_LEFT,
    SDLKey_SDLK_LEFTBRACKET, SDLKey_SDLK_LEFTPAREN, SDLKey_SDLK_LESS, SDLKey_SDLK_LMETA,
    SDLKey_SDLK_LSHIFT, SDLKey_SDLK_LSUPER, SDLKey_SDLK_MENU, SDLKey_SDLK_MINUS, SDLKey_SDLK_MODE,
    SDLKey_SDLK_NUMLOCK, SDLKey_SDLK_PAGEDOWN, SDLKey_SDLK_PAGEUP, SDLKey_SDLK_PAUSE,
    SDLKey_SDLK_PERIOD, SDLKey_SDLK_PLUS, SDLKey_SDLK_POWER, SDLKey_SDLK_PRINT,
    SDLKey_SDLK_QUESTION, SDLKey_SDLK_QUOTE, SDLKey_SDLK_QUOTEDBL, SDLKey_SDLK_RALT,
    SDLKey_SDLK_RCTRL, SDLKey_SDLK_RETURN, SDLKey_SDLK_RIGHT, SDLKey_SDLK_RIGHTBRACKET,
    SDLKey_SDLK_RIGHTPAREN, SDLKey_SDLK_RMETA, SDLKey_SDLK_RSHIFT, SDLKey_SDLK_RSUPER,
    SDLKey_SDLK_SCROLLOCK, SDLKey_SDLK_SEMICOLON, SDLKey_SDLK_SLASH, SDLKey_SDLK_SPACE,
    SDLKey_SDLK_SYSREQ, SDLKey_SDLK_TAB, SDLKey_SDLK_UNDERSCORE, SDLKey_SDLK_UNDO, SDLKey_SDLK_UP,
    SDLKey_SDLK_WORLD_0, SDLKey_SDLK_WORLD_1, SDLKey_SDLK_WORLD_10, SDLKey_SDLK_WORLD_11,
    SDLKey_SDLK_WORLD_12, SDLKey_SDLK_WORLD_13, SDLKey_SDLK_WORLD_14, SDLKey_SDLK_WORLD_15,
    SDLKey_SDLK_WORLD_16, SDLKey_SDLK_WORLD_17, SDLKey_SDLK_WORLD_18, SDLKey_SDLK_WORLD_19,
    SDLKey_SDLK_WORLD_2, SDLKey_SDLK_WORLD_20, SDLKey_SDLK_WORLD_21, SDLKey_SDLK_WORLD_22,
    SDLKey_SDLK_WORLD_23, SDLKey_SDLK_WORLD_24, SDLKey_SDLK_WORLD_25, SDLKey_SDLK_WORLD_26,
    SDLKey_SDLK_WORLD_27, SDLKey_SDLK_WORLD_28, SDLKey_SDLK_WORLD_29, SDLKey_SDLK_WORLD_3,
    SDLKey_SDLK_WORLD_30, SDLKey_SDLK_WORLD_31, SDLKey_SDLK_WORLD_32, SDLKey_SDLK_WORLD_33,
    SDLKey_SDLK_WORLD_34, SDLKey_SDLK_WORLD_35, SDLKey_SDLK_WORLD_36, SDLKey_SDLK_WORLD_37,
    SDLKey_SDLK_WORLD_38, SDLKey_SDLK_WORLD_39, SDLKey_SDLK_WORLD_4, SDLKey_SDLK_WORLD_40,
    SDLKey_SDLK_WORLD_41, SDLKey_SDLK_WORLD_42, SDLKey_SDLK_WORLD_43, SDLKey_SDLK_WORLD_44,
    SDLKey_SDLK_WORLD_45, SDLKey_SDLK_WORLD_46, SDLKey_SDLK_WORLD_47, SDLKey_SDLK_WORLD_48,
    SDLKey_SDLK_WORLD_49, SDLKey_SDLK_WORLD_5, SDLKey_SDLK_WORLD_50, SDLKey_SDLK_WORLD_51,
    SDLKey_SDLK_WORLD_52, SDLKey_SDLK_WORLD_53, SDLKey_SDLK_WORLD_54, SDLKey_SDLK_WORLD_55,
    SDLKey_SDLK_WORLD_56, SDLKey_SDLK_WORLD_57, SDLKey_SDLK_WORLD_58, SDLKey_SDLK_WORLD_59,
    SDLKey_SDLK_WORLD_6, SDLKey_SDLK_WORLD_60, SDLKey_SDLK_WORLD_61, SDLKey_SDLK_WORLD_62,
    SDLKey_SDLK_WORLD_63, SDLKey_SDLK_WORLD_64, SDLKey_SDLK_WORLD_65, SDLKey_SDLK_WORLD_66,
    SDLKey_SDLK_WORLD_67, SDLKey_SDLK_WORLD_68, SDLKey_SDLK_WORLD_69, SDLKey_SDLK_WORLD_7,
    SDLKey_SDLK_WORLD_70, SDLKey_SDLK_WORLD_71, SDLKey_SDLK_WORLD_72, SDLKey_SDLK_WORLD_73,
    SDLKey_SDLK_WORLD_74, SDLKey_SDLK_WORLD_75, SDLKey_SDLK_WORLD_76, SDLKey_SDLK_WORLD_77,
    SDLKey_SDLK_WORLD_78, SDLKey_SDLK_WORLD_79, SDLKey_SDLK_WORLD_8, SDLKey_SDLK_WORLD_80,
    SDLKey_SDLK_WORLD_81, SDLKey_SDLK_WORLD_82, SDLKey_SDLK_WORLD_83, SDLKey_SDLK_WORLD_84,
    SDLKey_SDLK_WORLD_85, SDLKey_SDLK_WORLD_86, SDLKey_SDLK_WORLD_87, SDLKey_SDLK_WORLD_88,
    SDLKey_SDLK_WORLD_89, SDLKey_SDLK_WORLD_9, SDLKey_SDLK_WORLD_90, SDLKey_SDLK_WORLD_91,
    SDLKey_SDLK_WORLD_92, SDLKey_SDLK_WORLD_93, SDLKey_SDLK_WORLD_94, SDLKey_SDLK_WORLD_95,
    SDLKey_SDLK_a, SDLKey_SDLK_b, SDLKey_SDLK_c, SDLKey_SDLK_d, SDLKey_SDLK_e, SDLKey_SDLK_f,
    SDLKey_SDLK_g, SDLKey_SDLK_h, SDLKey_SDLK_i, SDLKey_SDLK_j, SDLKey_SDLK_k, SDLKey_SDLK_l,
    SDLKey_SDLK_m, SDLKey_SDLK_n, SDLKey_SDLK_o, SDLKey_SDLK_p, SDLKey_SDLK_q, SDLKey_SDLK_r,
    SDLKey_SDLK_s, SDLKey_SDLK_t, SDLKey_SDLK_u, SDLKey_SDLK_v, SDLKey_SDLK_w, SDLKey_SDLK_x,
    SDLKey_SDLK_y, SDLKey_SDLK_z, SDLMod_KMOD_CAPS, SDLMod_KMOD_LALT, SDLMod_KMOD_LCTRL,
    SDLMod_KMOD_LMETA, SDLMod_KMOD_LSHIFT, SDLMod_KMOD_MODE, SDLMod_KMOD_NUM, SDLMod_KMOD_RALT,
    SDLMod_KMOD_RCTRL, SDLMod_KMOD_RESERVED, SDLMod_KMOD_RMETA, SDLMod_KMOD_RSHIFT, SDL_keysym,
};

use crate::convert;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeySym {
    /// hardware specific scancode
    pub scancode: u8,
    /// SDL virtual keysym
    pub symbol: Key,
    /// current key modifiers
    pub mod_: Mod,
    /// translated character
    pub unicode: u16,
}

impl KeySym {
    pub fn from_raw(raw: SDL_keysym) -> Result<Self, InvalidKeySym> {
        let symbol = Key::from_raw(raw.sym).map_err(|_| InvalidKeySym::Key)?;
        let mod_ = Mod::from_bits(raw.mod_).ok_or(InvalidKeySym::Mod)?;

        Ok(Self {
            scancode: raw.scancode,
            symbol,
            mod_,
            unicode: raw.unicode,
        })
    }

    #[must_use]
    pub fn to_raw(&self) -> SDL_keysym {
        SDL_keysym {
            scancode: self.scancode,
            sym: self.symbol as u32,
            mod_: self.mod_.bits(),
            unicode: self.unicode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid raw keysym")]
pub enum InvalidKeySym {
    #[error("invalid virtual keysym")]
    Key,

    #[error("invalid key modifier")]
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    First = convert::u32_to_isize(SDLKey_SDLK_FIRST),
    Backspace = convert::u32_to_isize(SDLKey_SDLK_BACKSPACE),
    Tab = convert::u32_to_isize(SDLKey_SDLK_TAB),
    Clear = convert::u32_to_isize(SDLKey_SDLK_CLEAR),
    Return = convert::u32_to_isize(SDLKey_SDLK_RETURN),
    Pause = convert::u32_to_isize(SDLKey_SDLK_PAUSE),
    Escape = convert::u32_to_isize(SDLKey_SDLK_ESCAPE),
    Space = convert::u32_to_isize(SDLKey_SDLK_SPACE),
    Exclaim = convert::u32_to_isize(SDLKey_SDLK_EXCLAIM),
    QuoteDbl = convert::u32_to_isize(SDLKey_SDLK_QUOTEDBL),
    Hash = convert::u32_to_isize(SDLKey_SDLK_HASH),
    Dollar = convert::u32_to_isize(SDLKey_SDLK_DOLLAR),
    Ampersand = convert::u32_to_isize(SDLKey_SDLK_AMPERSAND),
    Quote = convert::u32_to_isize(SDLKey_SDLK_QUOTE),
    LeftParen = convert::u32_to_isize(SDLKey_SDLK_LEFTPAREN),
    RightParen = convert::u32_to_isize(SDLKey_SDLK_RIGHTPAREN),
    Asterisk = convert::u32_to_isize(SDLKey_SDLK_ASTERISK),
    Plus = convert::u32_to_isize(SDLKey_SDLK_PLUS),
    Comma = convert::u32_to_isize(SDLKey_SDLK_COMMA),
    Minus = convert::u32_to_isize(SDLKey_SDLK_MINUS),
    Period = convert::u32_to_isize(SDLKey_SDLK_PERIOD),
    Slash = convert::u32_to_isize(SDLKey_SDLK_SLASH),
    Num0 = convert::u32_to_isize(SDLKey_SDLK_0),
    Num1 = convert::u32_to_isize(SDLKey_SDLK_1),
    Num2 = convert::u32_to_isize(SDLKey_SDLK_2),
    Num3 = convert::u32_to_isize(SDLKey_SDLK_3),
    Num4 = convert::u32_to_isize(SDLKey_SDLK_4),
    Num5 = convert::u32_to_isize(SDLKey_SDLK_5),
    Num6 = convert::u32_to_isize(SDLKey_SDLK_6),
    Num7 = convert::u32_to_isize(SDLKey_SDLK_7),
    Num8 = convert::u32_to_isize(SDLKey_SDLK_8),
    Num9 = convert::u32_to_isize(SDLKey_SDLK_9),
    Colon = convert::u32_to_isize(SDLKey_SDLK_COLON),
    Semicolon = convert::u32_to_isize(SDLKey_SDLK_SEMICOLON),
    Less = convert::u32_to_isize(SDLKey_SDLK_LESS),
    Equals = convert::u32_to_isize(SDLKey_SDLK_EQUALS),
    Greater = convert::u32_to_isize(SDLKey_SDLK_GREATER),
    Question = convert::u32_to_isize(SDLKey_SDLK_QUESTION),
    At = convert::u32_to_isize(SDLKey_SDLK_AT),
    LeftBracket = convert::u32_to_isize(SDLKey_SDLK_LEFTBRACKET),
    Backslash = convert::u32_to_isize(SDLKey_SDLK_BACKSLASH),
    RightBracket = convert::u32_to_isize(SDLKey_SDLK_RIGHTBRACKET),
    Caret = convert::u32_to_isize(SDLKey_SDLK_CARET),
    Underscore = convert::u32_to_isize(SDLKey_SDLK_UNDERSCORE),
    Backquote = convert::u32_to_isize(SDLKey_SDLK_BACKQUOTE),
    A = convert::u32_to_isize(SDLKey_SDLK_a),
    B = convert::u32_to_isize(SDLKey_SDLK_b),
    C = convert::u32_to_isize(SDLKey_SDLK_c),
    D = convert::u32_to_isize(SDLKey_SDLK_d),
    E = convert::u32_to_isize(SDLKey_SDLK_e),
    F = convert::u32_to_isize(SDLKey_SDLK_f),
    G = convert::u32_to_isize(SDLKey_SDLK_g),
    H = convert::u32_to_isize(SDLKey_SDLK_h),
    I = convert::u32_to_isize(SDLKey_SDLK_i),
    J = convert::u32_to_isize(SDLKey_SDLK_j),
    K = convert::u32_to_isize(SDLKey_SDLK_k),
    L = convert::u32_to_isize(SDLKey_SDLK_l),
    M = convert::u32_to_isize(SDLKey_SDLK_m),
    N = convert::u32_to_isize(SDLKey_SDLK_n),
    O = convert::u32_to_isize(SDLKey_SDLK_o),
    P = convert::u32_to_isize(SDLKey_SDLK_p),
    Q = convert::u32_to_isize(SDLKey_SDLK_q),
    R = convert::u32_to_isize(SDLKey_SDLK_r),
    S = convert::u32_to_isize(SDLKey_SDLK_s),
    T = convert::u32_to_isize(SDLKey_SDLK_t),
    U = convert::u32_to_isize(SDLKey_SDLK_u),
    V = convert::u32_to_isize(SDLKey_SDLK_v),
    W = convert::u32_to_isize(SDLKey_SDLK_w),
    X = convert::u32_to_isize(SDLKey_SDLK_x),
    Y = convert::u32_to_isize(SDLKey_SDLK_y),
    Z = convert::u32_to_isize(SDLKey_SDLK_z),
    Delete = convert::u32_to_isize(SDLKey_SDLK_DELETE),
    World0 = convert::u32_to_isize(SDLKey_SDLK_WORLD_0),
    World1 = convert::u32_to_isize(SDLKey_SDLK_WORLD_1),
    World2 = convert::u32_to_isize(SDLKey_SDLK_WORLD_2),
    World3 = convert::u32_to_isize(SDLKey_SDLK_WORLD_3),
    World4 = convert::u32_to_isize(SDLKey_SDLK_WORLD_4),
    World5 = convert::u32_to_isize(SDLKey_SDLK_WORLD_5),
    World6 = convert::u32_to_isize(SDLKey_SDLK_WORLD_6),
    World7 = convert::u32_to_isize(SDLKey_SDLK_WORLD_7),
    World8 = convert::u32_to_isize(SDLKey_SDLK_WORLD_8),
    World9 = convert::u32_to_isize(SDLKey_SDLK_WORLD_9),
    World10 = convert::u32_to_isize(SDLKey_SDLK_WORLD_10),
    World11 = convert::u32_to_isize(SDLKey_SDLK_WORLD_11),
    World12 = convert::u32_to_isize(SDLKey_SDLK_WORLD_12),
    World13 = convert::u32_to_isize(SDLKey_SDLK_WORLD_13),
    World14 = convert::u32_to_isize(SDLKey_SDLK_WORLD_14),
    World15 = convert::u32_to_isize(SDLKey_SDLK_WORLD_15),
    World16 = convert::u32_to_isize(SDLKey_SDLK_WORLD_16),
    World17 = convert::u32_to_isize(SDLKey_SDLK_WORLD_17),
    World18 = convert::u32_to_isize(SDLKey_SDLK_WORLD_18),
    World19 = convert::u32_to_isize(SDLKey_SDLK_WORLD_19),
    World20 = convert::u32_to_isize(SDLKey_SDLK_WORLD_20),
    World21 = convert::u32_to_isize(SDLKey_SDLK_WORLD_21),
    World22 = convert::u32_to_isize(SDLKey_SDLK_WORLD_22),
    World23 = convert::u32_to_isize(SDLKey_SDLK_WORLD_23),
    World24 = convert::u32_to_isize(SDLKey_SDLK_WORLD_24),
    World25 = convert::u32_to_isize(SDLKey_SDLK_WORLD_25),
    World26 = convert::u32_to_isize(SDLKey_SDLK_WORLD_26),
    World27 = convert::u32_to_isize(SDLKey_SDLK_WORLD_27),
    World28 = convert::u32_to_isize(SDLKey_SDLK_WORLD_28),
    World29 = convert::u32_to_isize(SDLKey_SDLK_WORLD_29),
    World30 = convert::u32_to_isize(SDLKey_SDLK_WORLD_30),
    World31 = convert::u32_to_isize(SDLKey_SDLK_WORLD_31),
    World32 = convert::u32_to_isize(SDLKey_SDLK_WORLD_32),
    World33 = convert::u32_to_isize(SDLKey_SDLK_WORLD_33),
    World34 = convert::u32_to_isize(SDLKey_SDLK_WORLD_34),
    World35 = convert::u32_to_isize(SDLKey_SDLK_WORLD_35),
    World36 = convert::u32_to_isize(SDLKey_SDLK_WORLD_36),
    World37 = convert::u32_to_isize(SDLKey_SDLK_WORLD_37),
    World38 = convert::u32_to_isize(SDLKey_SDLK_WORLD_38),
    World39 = convert::u32_to_isize(SDLKey_SDLK_WORLD_39),
    World40 = convert::u32_to_isize(SDLKey_SDLK_WORLD_40),
    World41 = convert::u32_to_isize(SDLKey_SDLK_WORLD_41),
    World42 = convert::u32_to_isize(SDLKey_SDLK_WORLD_42),
    World43 = convert::u32_to_isize(SDLKey_SDLK_WORLD_43),
    World44 = convert::u32_to_isize(SDLKey_SDLK_WORLD_44),
    World45 = convert::u32_to_isize(SDLKey_SDLK_WORLD_45),
    World46 = convert::u32_to_isize(SDLKey_SDLK_WORLD_46),
    World47 = convert::u32_to_isize(SDLKey_SDLK_WORLD_47),
    World48 = convert::u32_to_isize(SDLKey_SDLK_WORLD_48),
    World49 = convert::u32_to_isize(SDLKey_SDLK_WORLD_49),
    World50 = convert::u32_to_isize(SDLKey_SDLK_WORLD_50),
    World51 = convert::u32_to_isize(SDLKey_SDLK_WORLD_51),
    World52 = convert::u32_to_isize(SDLKey_SDLK_WORLD_52),
    World53 = convert::u32_to_isize(SDLKey_SDLK_WORLD_53),
    World54 = convert::u32_to_isize(SDLKey_SDLK_WORLD_54),
    World55 = convert::u32_to_isize(SDLKey_SDLK_WORLD_55),
    World56 = convert::u32_to_isize(SDLKey_SDLK_WORLD_56),
    World57 = convert::u32_to_isize(SDLKey_SDLK_WORLD_57),
    World58 = convert::u32_to_isize(SDLKey_SDLK_WORLD_58),
    World59 = convert::u32_to_isize(SDLKey_SDLK_WORLD_59),
    World60 = convert::u32_to_isize(SDLKey_SDLK_WORLD_60),
    World61 = convert::u32_to_isize(SDLKey_SDLK_WORLD_61),
    World62 = convert::u32_to_isize(SDLKey_SDLK_WORLD_62),
    World63 = convert::u32_to_isize(SDLKey_SDLK_WORLD_63),
    World64 = convert::u32_to_isize(SDLKey_SDLK_WORLD_64),
    World65 = convert::u32_to_isize(SDLKey_SDLK_WORLD_65),
    World66 = convert::u32_to_isize(SDLKey_SDLK_WORLD_66),
    World67 = convert::u32_to_isize(SDLKey_SDLK_WORLD_67),
    World68 = convert::u32_to_isize(SDLKey_SDLK_WORLD_68),
    World69 = convert::u32_to_isize(SDLKey_SDLK_WORLD_69),
    World70 = convert::u32_to_isize(SDLKey_SDLK_WORLD_70),
    World71 = convert::u32_to_isize(SDLKey_SDLK_WORLD_71),
    World72 = convert::u32_to_isize(SDLKey_SDLK_WORLD_72),
    World73 = convert::u32_to_isize(SDLKey_SDLK_WORLD_73),
    World74 = convert::u32_to_isize(SDLKey_SDLK_WORLD_74),
    World75 = convert::u32_to_isize(SDLKey_SDLK_WORLD_75),
    World76 = convert::u32_to_isize(SDLKey_SDLK_WORLD_76),
    World77 = convert::u32_to_isize(SDLKey_SDLK_WORLD_77),
    World78 = convert::u32_to_isize(SDLKey_SDLK_WORLD_78),
    World79 = convert::u32_to_isize(SDLKey_SDLK_WORLD_79),
    World80 = convert::u32_to_isize(SDLKey_SDLK_WORLD_80),
    World81 = convert::u32_to_isize(SDLKey_SDLK_WORLD_81),
    World82 = convert::u32_to_isize(SDLKey_SDLK_WORLD_82),
    World83 = convert::u32_to_isize(SDLKey_SDLK_WORLD_83),
    World84 = convert::u32_to_isize(SDLKey_SDLK_WORLD_84),
    World85 = convert::u32_to_isize(SDLKey_SDLK_WORLD_85),
    World86 = convert::u32_to_isize(SDLKey_SDLK_WORLD_86),
    World87 = convert::u32_to_isize(SDLKey_SDLK_WORLD_87),
    World88 = convert::u32_to_isize(SDLKey_SDLK_WORLD_88),
    World89 = convert::u32_to_isize(SDLKey_SDLK_WORLD_89),
    World90 = convert::u32_to_isize(SDLKey_SDLK_WORLD_90),
    World91 = convert::u32_to_isize(SDLKey_SDLK_WORLD_91),
    World92 = convert::u32_to_isize(SDLKey_SDLK_WORLD_92),
    World93 = convert::u32_to_isize(SDLKey_SDLK_WORLD_93),
    World94 = convert::u32_to_isize(SDLKey_SDLK_WORLD_94),
    World95 = convert::u32_to_isize(SDLKey_SDLK_WORLD_95),
    KeyPad0 = convert::u32_to_isize(SDLKey_SDLK_KP0),
    KeyPad1 = convert::u32_to_isize(SDLKey_SDLK_KP1),
    KeyPad2 = convert::u32_to_isize(SDLKey_SDLK_KP2),
    KeyPad3 = convert::u32_to_isize(SDLKey_SDLK_KP3),
    KeyPad4 = convert::u32_to_isize(SDLKey_SDLK_KP4),
    KeyPad5 = convert::u32_to_isize(SDLKey_SDLK_KP5),
    KeyPad6 = convert::u32_to_isize(SDLKey_SDLK_KP6),
    KeyPad7 = convert::u32_to_isize(SDLKey_SDLK_KP7),
    KeyPad8 = convert::u32_to_isize(SDLKey_SDLK_KP8),
    KeyPad9 = convert::u32_to_isize(SDLKey_SDLK_KP9),
    KeyPadPeriod = convert::u32_to_isize(SDLKey_SDLK_KP_PERIOD),
    KeyPadDivide = convert::u32_to_isize(SDLKey_SDLK_KP_DIVIDE),
    KeyPadMultiply = convert::u32_to_isize(SDLKey_SDLK_KP_MULTIPLY),
    KeyPadMinus = convert::u32_to_isize(SDLKey_SDLK_KP_MINUS),
    KeyPadPlus = convert::u32_to_isize(SDLKey_SDLK_KP_PLUS),
    KeyPadEnter = convert::u32_to_isize(SDLKey_SDLK_KP_ENTER),
    KeyPadEquals = convert::u32_to_isize(SDLKey_SDLK_KP_EQUALS),
    Up = convert::u32_to_isize(SDLKey_SDLK_UP),
    Down = convert::u32_to_isize(SDLKey_SDLK_DOWN),
    Right = convert::u32_to_isize(SDLKey_SDLK_RIGHT),
    Left = convert::u32_to_isize(SDLKey_SDLK_LEFT),
    Insert = convert::u32_to_isize(SDLKey_SDLK_INSERT),
    Home = convert::u32_to_isize(SDLKey_SDLK_HOME),
    End = convert::u32_to_isize(SDLKey_SDLK_END),
    Pageup = convert::u32_to_isize(SDLKey_SDLK_PAGEUP),
    Pagedown = convert::u32_to_isize(SDLKey_SDLK_PAGEDOWN),
    F1 = convert::u32_to_isize(SDLKey_SDLK_F1),
    F2 = convert::u32_to_isize(SDLKey_SDLK_F2),
    F3 = convert::u32_to_isize(SDLKey_SDLK_F3),
    F4 = convert::u32_to_isize(SDLKey_SDLK_F4),
    F5 = convert::u32_to_isize(SDLKey_SDLK_F5),
    F6 = convert::u32_to_isize(SDLKey_SDLK_F6),
    F7 = convert::u32_to_isize(SDLKey_SDLK_F7),
    F8 = convert::u32_to_isize(SDLKey_SDLK_F8),
    F9 = convert::u32_to_isize(SDLKey_SDLK_F9),
    F10 = convert::u32_to_isize(SDLKey_SDLK_F10),
    F11 = convert::u32_to_isize(SDLKey_SDLK_F11),
    F12 = convert::u32_to_isize(SDLKey_SDLK_F12),
    F13 = convert::u32_to_isize(SDLKey_SDLK_F13),
    F14 = convert::u32_to_isize(SDLKey_SDLK_F14),
    F15 = convert::u32_to_isize(SDLKey_SDLK_F15),
    Numlock = convert::u32_to_isize(SDLKey_SDLK_NUMLOCK),
    Capslock = convert::u32_to_isize(SDLKey_SDLK_CAPSLOCK),
    Scrollock = convert::u32_to_isize(SDLKey_SDLK_SCROLLOCK),
    RightShift = convert::u32_to_isize(SDLKey_SDLK_RSHIFT),
    LeftShift = convert::u32_to_isize(SDLKey_SDLK_LSHIFT),
    RightCtrl = convert::u32_to_isize(SDLKey_SDLK_RCTRL),
    LeftCtrl = convert::u32_to_isize(SDLKey_SDLK_LCTRL),
    RightAlt = convert::u32_to_isize(SDLKey_SDLK_RALT),
    LeftAlt = convert::u32_to_isize(SDLKey_SDLK_LALT),
    RightMeta = convert::u32_to_isize(SDLKey_SDLK_RMETA),
    LeftMeta = convert::u32_to_isize(SDLKey_SDLK_LMETA),
    LeftSuper = convert::u32_to_isize(SDLKey_SDLK_LSUPER),
    RightSuper = convert::u32_to_isize(SDLKey_SDLK_RSUPER),
    Mode = convert::u32_to_isize(SDLKey_SDLK_MODE),
    Compose = convert::u32_to_isize(SDLKey_SDLK_COMPOSE),
    Help = convert::u32_to_isize(SDLKey_SDLK_HELP),
    Print = convert::u32_to_isize(SDLKey_SDLK_PRINT),
    Sysreq = convert::u32_to_isize(SDLKey_SDLK_SYSREQ),
    Break = convert::u32_to_isize(SDLKey_SDLK_BREAK),
    Menu = convert::u32_to_isize(SDLKey_SDLK_MENU),
    Power = convert::u32_to_isize(SDLKey_SDLK_POWER),
    Euro = convert::u32_to_isize(SDLKey_SDLK_EURO),
    Undo = convert::u32_to_isize(SDLKey_SDLK_UNDO),
}

impl Key {
    #[allow(clippy::too_many_lines)]
    fn from_raw(key: u32) -> Result<Self, InvalidKey> {
        use Key as K;
        Ok(match key {
            sdl_sys::SDLKey_SDLK_FIRST => K::First,
            sdl_sys::SDLKey_SDLK_BACKSPACE => K::Backspace,
            sdl_sys::SDLKey_SDLK_TAB => K::Tab,
            sdl_sys::SDLKey_SDLK_CLEAR => K::Clear,
            sdl_sys::SDLKey_SDLK_RETURN => K::Return,
            sdl_sys::SDLKey_SDLK_PAUSE => K::Pause,
            sdl_sys::SDLKey_SDLK_ESCAPE => K::Escape,
            sdl_sys::SDLKey_SDLK_SPACE => K::Space,
            sdl_sys::SDLKey_SDLK_EXCLAIM => K::Exclaim,
            sdl_sys::SDLKey_SDLK_QUOTEDBL => K::QuoteDbl,
            sdl_sys::SDLKey_SDLK_HASH => K::Hash,
            sdl_sys::SDLKey_SDLK_DOLLAR => K::Dollar,
            sdl_sys::SDLKey_SDLK_AMPERSAND => K::Ampersand,
            sdl_sys::SDLKey_SDLK_QUOTE => K::Quote,
            sdl_sys::SDLKey_SDLK_LEFTPAREN => K::LeftParen,
            sdl_sys::SDLKey_SDLK_RIGHTPAREN => K::RightParen,
            sdl_sys::SDLKey_SDLK_ASTERISK => K::Asterisk,
            sdl_sys::SDLKey_SDLK_PLUS => K::Plus,
            sdl_sys::SDLKey_SDLK_COMMA => K::Comma,
            sdl_sys::SDLKey_SDLK_MINUS => K::Minus,
            sdl_sys::SDLKey_SDLK_PERIOD => K::Period,
            sdl_sys::SDLKey_SDLK_SLASH => K::Slash,
            sdl_sys::SDLKey_SDLK_0 => K::Num0,
            sdl_sys::SDLKey_SDLK_1 => K::Num1,
            sdl_sys::SDLKey_SDLK_2 => K::Num2,
            sdl_sys::SDLKey_SDLK_3 => K::Num3,
            sdl_sys::SDLKey_SDLK_4 => K::Num4,
            sdl_sys::SDLKey_SDLK_5 => K::Num5,
            sdl_sys::SDLKey_SDLK_6 => K::Num6,
            sdl_sys::SDLKey_SDLK_7 => K::Num7,
            sdl_sys::SDLKey_SDLK_8 => K::Num8,
            sdl_sys::SDLKey_SDLK_9 => K::Num9,
            sdl_sys::SDLKey_SDLK_COLON => K::Colon,
            sdl_sys::SDLKey_SDLK_SEMICOLON => K::Semicolon,
            sdl_sys::SDLKey_SDLK_LESS => K::Less,
            sdl_sys::SDLKey_SDLK_EQUALS => K::Equals,
            sdl_sys::SDLKey_SDLK_GREATER => K::Greater,
            sdl_sys::SDLKey_SDLK_QUESTION => K::Question,
            sdl_sys::SDLKey_SDLK_AT => K::At,
            sdl_sys::SDLKey_SDLK_LEFTBRACKET => K::LeftBracket,
            sdl_sys::SDLKey_SDLK_BACKSLASH => K::Backslash,
            sdl_sys::SDLKey_SDLK_RIGHTBRACKET => K::RightBracket,
            sdl_sys::SDLKey_SDLK_CARET => K::Caret,
            sdl_sys::SDLKey_SDLK_UNDERSCORE => K::Underscore,
            sdl_sys::SDLKey_SDLK_BACKQUOTE => K::Backquote,
            sdl_sys::SDLKey_SDLK_a => K::A,
            sdl_sys::SDLKey_SDLK_b => K::B,
            sdl_sys::SDLKey_SDLK_c => K::C,
            sdl_sys::SDLKey_SDLK_d => K::D,
            sdl_sys::SDLKey_SDLK_e => K::E,
            sdl_sys::SDLKey_SDLK_f => K::F,
            sdl_sys::SDLKey_SDLK_g => K::G,
            sdl_sys::SDLKey_SDLK_h => K::H,
            sdl_sys::SDLKey_SDLK_i => K::I,
            sdl_sys::SDLKey_SDLK_j => K::J,
            sdl_sys::SDLKey_SDLK_k => K::K,
            sdl_sys::SDLKey_SDLK_l => K::L,
            sdl_sys::SDLKey_SDLK_m => K::M,
            sdl_sys::SDLKey_SDLK_n => K::N,
            sdl_sys::SDLKey_SDLK_o => K::O,
            sdl_sys::SDLKey_SDLK_p => K::P,
            sdl_sys::SDLKey_SDLK_q => K::Q,
            sdl_sys::SDLKey_SDLK_r => K::R,
            sdl_sys::SDLKey_SDLK_s => K::S,
            sdl_sys::SDLKey_SDLK_t => K::T,
            sdl_sys::SDLKey_SDLK_u => K::U,
            sdl_sys::SDLKey_SDLK_v => K::V,
            sdl_sys::SDLKey_SDLK_w => K::W,
            sdl_sys::SDLKey_SDLK_x => K::X,
            sdl_sys::SDLKey_SDLK_y => K::Y,
            sdl_sys::SDLKey_SDLK_z => K::Z,
            sdl_sys::SDLKey_SDLK_DELETE => K::Delete,
            sdl_sys::SDLKey_SDLK_WORLD_0 => K::World0,
            sdl_sys::SDLKey_SDLK_WORLD_1 => K::World1,
            sdl_sys::SDLKey_SDLK_WORLD_2 => K::World2,
            sdl_sys::SDLKey_SDLK_WORLD_3 => K::World3,
            sdl_sys::SDLKey_SDLK_WORLD_4 => K::World4,
            sdl_sys::SDLKey_SDLK_WORLD_5 => K::World5,
            sdl_sys::SDLKey_SDLK_WORLD_6 => K::World6,
            sdl_sys::SDLKey_SDLK_WORLD_7 => K::World7,
            sdl_sys::SDLKey_SDLK_WORLD_8 => K::World8,
            sdl_sys::SDLKey_SDLK_WORLD_9 => K::World9,
            sdl_sys::SDLKey_SDLK_WORLD_10 => K::World10,
            sdl_sys::SDLKey_SDLK_WORLD_11 => K::World11,
            sdl_sys::SDLKey_SDLK_WORLD_12 => K::World12,
            sdl_sys::SDLKey_SDLK_WORLD_13 => K::World13,
            sdl_sys::SDLKey_SDLK_WORLD_14 => K::World14,
            sdl_sys::SDLKey_SDLK_WORLD_15 => K::World15,
            sdl_sys::SDLKey_SDLK_WORLD_16 => K::World16,
            sdl_sys::SDLKey_SDLK_WORLD_17 => K::World17,
            sdl_sys::SDLKey_SDLK_WORLD_18 => K::World18,
            sdl_sys::SDLKey_SDLK_WORLD_19 => K::World19,
            sdl_sys::SDLKey_SDLK_WORLD_20 => K::World20,
            sdl_sys::SDLKey_SDLK_WORLD_21 => K::World21,
            sdl_sys::SDLKey_SDLK_WORLD_22 => K::World22,
            sdl_sys::SDLKey_SDLK_WORLD_23 => K::World23,
            sdl_sys::SDLKey_SDLK_WORLD_24 => K::World24,
            sdl_sys::SDLKey_SDLK_WORLD_25 => K::World25,
            sdl_sys::SDLKey_SDLK_WORLD_26 => K::World26,
            sdl_sys::SDLKey_SDLK_WORLD_27 => K::World27,
            sdl_sys::SDLKey_SDLK_WORLD_28 => K::World28,
            sdl_sys::SDLKey_SDLK_WORLD_29 => K::World29,
            sdl_sys::SDLKey_SDLK_WORLD_30 => K::World30,
            sdl_sys::SDLKey_SDLK_WORLD_31 => K::World31,
            sdl_sys::SDLKey_SDLK_WORLD_32 => K::World32,
            sdl_sys::SDLKey_SDLK_WORLD_33 => K::World33,
            sdl_sys::SDLKey_SDLK_WORLD_34 => K::World34,
            sdl_sys::SDLKey_SDLK_WORLD_35 => K::World35,
            sdl_sys::SDLKey_SDLK_WORLD_36 => K::World36,
            sdl_sys::SDLKey_SDLK_WORLD_37 => K::World37,
            sdl_sys::SDLKey_SDLK_WORLD_38 => K::World38,
            sdl_sys::SDLKey_SDLK_WORLD_39 => K::World39,
            sdl_sys::SDLKey_SDLK_WORLD_40 => K::World40,
            sdl_sys::SDLKey_SDLK_WORLD_41 => K::World41,
            sdl_sys::SDLKey_SDLK_WORLD_42 => K::World42,
            sdl_sys::SDLKey_SDLK_WORLD_43 => K::World43,
            sdl_sys::SDLKey_SDLK_WORLD_44 => K::World44,
            sdl_sys::SDLKey_SDLK_WORLD_45 => K::World45,
            sdl_sys::SDLKey_SDLK_WORLD_46 => K::World46,
            sdl_sys::SDLKey_SDLK_WORLD_47 => K::World47,
            sdl_sys::SDLKey_SDLK_WORLD_48 => K::World48,
            sdl_sys::SDLKey_SDLK_WORLD_49 => K::World49,
            sdl_sys::SDLKey_SDLK_WORLD_50 => K::World50,
            sdl_sys::SDLKey_SDLK_WORLD_51 => K::World51,
            sdl_sys::SDLKey_SDLK_WORLD_52 => K::World52,
            sdl_sys::SDLKey_SDLK_WORLD_53 => K::World53,
            sdl_sys::SDLKey_SDLK_WORLD_54 => K::World54,
            sdl_sys::SDLKey_SDLK_WORLD_55 => K::World55,
            sdl_sys::SDLKey_SDLK_WORLD_56 => K::World56,
            sdl_sys::SDLKey_SDLK_WORLD_57 => K::World57,
            sdl_sys::SDLKey_SDLK_WORLD_58 => K::World58,
            sdl_sys::SDLKey_SDLK_WORLD_59 => K::World59,
            sdl_sys::SDLKey_SDLK_WORLD_60 => K::World60,
            sdl_sys::SDLKey_SDLK_WORLD_61 => K::World61,
            sdl_sys::SDLKey_SDLK_WORLD_62 => K::World62,
            sdl_sys::SDLKey_SDLK_WORLD_63 => K::World63,
            sdl_sys::SDLKey_SDLK_WORLD_64 => K::World64,
            sdl_sys::SDLKey_SDLK_WORLD_65 => K::World65,
            sdl_sys::SDLKey_SDLK_WORLD_66 => K::World66,
            sdl_sys::SDLKey_SDLK_WORLD_67 => K::World67,
            sdl_sys::SDLKey_SDLK_WORLD_68 => K::World68,
            sdl_sys::SDLKey_SDLK_WORLD_69 => K::World69,
            sdl_sys::SDLKey_SDLK_WORLD_70 => K::World70,
            sdl_sys::SDLKey_SDLK_WORLD_71 => K::World71,
            sdl_sys::SDLKey_SDLK_WORLD_72 => K::World72,
            sdl_sys::SDLKey_SDLK_WORLD_73 => K::World73,
            sdl_sys::SDLKey_SDLK_WORLD_74 => K::World74,
            sdl_sys::SDLKey_SDLK_WORLD_75 => K::World75,
            sdl_sys::SDLKey_SDLK_WORLD_76 => K::World76,
            sdl_sys::SDLKey_SDLK_WORLD_77 => K::World77,
            sdl_sys::SDLKey_SDLK_WORLD_78 => K::World78,
            sdl_sys::SDLKey_SDLK_WORLD_79 => K::World79,
            sdl_sys::SDLKey_SDLK_WORLD_80 => K::World80,
            sdl_sys::SDLKey_SDLK_WORLD_81 => K::World81,
            sdl_sys::SDLKey_SDLK_WORLD_82 => K::World82,
            sdl_sys::SDLKey_SDLK_WORLD_83 => K::World83,
            sdl_sys::SDLKey_SDLK_WORLD_84 => K::World84,
            sdl_sys::SDLKey_SDLK_WORLD_85 => K::World85,
            sdl_sys::SDLKey_SDLK_WORLD_86 => K::World86,
            sdl_sys::SDLKey_SDLK_WORLD_87 => K::World87,
            sdl_sys::SDLKey_SDLK_WORLD_88 => K::World88,
            sdl_sys::SDLKey_SDLK_WORLD_89 => K::World89,
            sdl_sys::SDLKey_SDLK_WORLD_90 => K::World90,
            sdl_sys::SDLKey_SDLK_WORLD_91 => K::World91,
            sdl_sys::SDLKey_SDLK_WORLD_92 => K::World92,
            sdl_sys::SDLKey_SDLK_WORLD_93 => K::World93,
            sdl_sys::SDLKey_SDLK_WORLD_94 => K::World94,
            sdl_sys::SDLKey_SDLK_WORLD_95 => K::World95,
            sdl_sys::SDLKey_SDLK_KP0 => K::KeyPad0,
            sdl_sys::SDLKey_SDLK_KP1 => K::KeyPad1,
            sdl_sys::SDLKey_SDLK_KP2 => K::KeyPad2,
            sdl_sys::SDLKey_SDLK_KP3 => K::KeyPad3,
            sdl_sys::SDLKey_SDLK_KP4 => K::KeyPad4,
            sdl_sys::SDLKey_SDLK_KP5 => K::KeyPad5,
            sdl_sys::SDLKey_SDLK_KP6 => K::KeyPad6,
            sdl_sys::SDLKey_SDLK_KP7 => K::KeyPad7,
            sdl_sys::SDLKey_SDLK_KP8 => K::KeyPad8,
            sdl_sys::SDLKey_SDLK_KP9 => K::KeyPad9,
            sdl_sys::SDLKey_SDLK_KP_PERIOD => K::KeyPadPeriod,
            sdl_sys::SDLKey_SDLK_KP_DIVIDE => K::KeyPadDivide,
            sdl_sys::SDLKey_SDLK_KP_MULTIPLY => K::KeyPadMultiply,
            sdl_sys::SDLKey_SDLK_KP_MINUS => K::KeyPadMinus,
            sdl_sys::SDLKey_SDLK_KP_PLUS => K::KeyPadPlus,
            sdl_sys::SDLKey_SDLK_KP_ENTER => K::KeyPadEnter,
            sdl_sys::SDLKey_SDLK_KP_EQUALS => K::KeyPadEquals,
            sdl_sys::SDLKey_SDLK_UP => K::Up,
            sdl_sys::SDLKey_SDLK_DOWN => K::Down,
            sdl_sys::SDLKey_SDLK_RIGHT => K::Right,
            sdl_sys::SDLKey_SDLK_LEFT => K::Left,
            sdl_sys::SDLKey_SDLK_INSERT => K::Insert,
            sdl_sys::SDLKey_SDLK_HOME => K::Home,
            sdl_sys::SDLKey_SDLK_END => K::End,
            sdl_sys::SDLKey_SDLK_PAGEUP => K::Pageup,
            sdl_sys::SDLKey_SDLK_PAGEDOWN => K::Pagedown,
            sdl_sys::SDLKey_SDLK_F1 => K::F1,
            sdl_sys::SDLKey_SDLK_F2 => K::F2,
            sdl_sys::SDLKey_SDLK_F3 => K::F3,
            sdl_sys::SDLKey_SDLK_F4 => K::F4,
            sdl_sys::SDLKey_SDLK_F5 => K::F5,
            sdl_sys::SDLKey_SDLK_F6 => K::F6,
            sdl_sys::SDLKey_SDLK_F7 => K::F7,
            sdl_sys::SDLKey_SDLK_F8 => K::F8,
            sdl_sys::SDLKey_SDLK_F9 => K::F9,
            sdl_sys::SDLKey_SDLK_F10 => K::F10,
            sdl_sys::SDLKey_SDLK_F11 => K::F11,
            sdl_sys::SDLKey_SDLK_F12 => K::F12,
            sdl_sys::SDLKey_SDLK_F13 => K::F13,
            sdl_sys::SDLKey_SDLK_F14 => K::F14,
            sdl_sys::SDLKey_SDLK_F15 => K::F15,
            sdl_sys::SDLKey_SDLK_NUMLOCK => K::Numlock,
            sdl_sys::SDLKey_SDLK_CAPSLOCK => K::Capslock,
            sdl_sys::SDLKey_SDLK_SCROLLOCK => K::Scrollock,
            sdl_sys::SDLKey_SDLK_RSHIFT => K::RightShift,
            sdl_sys::SDLKey_SDLK_LSHIFT => K::LeftShift,
            sdl_sys::SDLKey_SDLK_RCTRL => K::RightCtrl,
            sdl_sys::SDLKey_SDLK_LCTRL => K::LeftCtrl,
            sdl_sys::SDLKey_SDLK_RALT => K::RightAlt,
            sdl_sys::SDLKey_SDLK_LALT => K::LeftAlt,
            sdl_sys::SDLKey_SDLK_RMETA => K::RightMeta,
            sdl_sys::SDLKey_SDLK_LMETA => K::LeftMeta,
            sdl_sys::SDLKey_SDLK_LSUPER => K::LeftSuper,
            sdl_sys::SDLKey_SDLK_RSUPER => K::RightSuper,
            sdl_sys::SDLKey_SDLK_MODE => K::Mode,
            sdl_sys::SDLKey_SDLK_COMPOSE => K::Compose,
            sdl_sys::SDLKey_SDLK_HELP => K::Help,
            sdl_sys::SDLKey_SDLK_PRINT => K::Print,
            sdl_sys::SDLKey_SDLK_SYSREQ => K::Sysreq,
            sdl_sys::SDLKey_SDLK_BREAK => K::Break,
            sdl_sys::SDLKey_SDLK_MENU => K::Menu,
            sdl_sys::SDLKey_SDLK_POWER => K::Power,
            sdl_sys::SDLKey_SDLK_EURO => K::Euro,
            sdl_sys::SDLKey_SDLK_UNDO => K::Undo,
            _ => return Err(InvalidKey),
        })
    }

    #[inline]
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self as u16
    }

    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid virtual key")]
pub struct InvalidKey;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Mod: u32 {
        const LEFT_SHIFT = SDLMod_KMOD_LSHIFT;
        const RIGHT_SHIFT = SDLMod_KMOD_RSHIFT;
        const LEFT_CTRL = SDLMod_KMOD_LCTRL;
        const RIGHT_CTRL = SDLMod_KMOD_RCTRL;
        const LEFT_ALT = SDLMod_KMOD_LALT;
        const RIGHT_ALT = SDLMod_KMOD_RALT;
        const LEFT_META = SDLMod_KMOD_LMETA;
        const RIGHT_META = SDLMod_KMOD_RMETA;
        const NUM = SDLMod_KMOD_NUM;
        const CAPS = SDLMod_KMOD_CAPS;
        const MODE = SDLMod_KMOD_MODE;
        const RESERVED = SDLMod_KMOD_RESERVED;
    }
}

impl Mod {
    #[must_use]
    pub fn is_shift(&self) -> bool {
        self.intersects(Self::LEFT_SHIFT | Self::RIGHT_SHIFT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid key modifier")]
pub struct InvalidMod;
