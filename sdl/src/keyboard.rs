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
    SDLMod_KMOD_LMETA, SDLMod_KMOD_LSHIFT, SDLMod_KMOD_MODE, SDLMod_KMOD_NONE, SDLMod_KMOD_NUM,
    SDLMod_KMOD_RALT, SDLMod_KMOD_RCTRL, SDLMod_KMOD_RMETA, SDLMod_KMOD_RSHIFT, SDL_keysym,
};

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
        let mod_ = Mod::from_raw(raw.mod_).map_err(|_| InvalidKeySym::Mod)?;

        Ok(Self {
            scancode: raw.scancode,
            symbol,
            mod_,
            unicode: raw.unicode,
        })
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
    First = SDLKey_SDLK_FIRST as isize,
    Backspace = SDLKey_SDLK_BACKSPACE as isize,
    Tab = SDLKey_SDLK_TAB as isize,
    Clear = SDLKey_SDLK_CLEAR as isize,
    Return = SDLKey_SDLK_RETURN as isize,
    Pause = SDLKey_SDLK_PAUSE as isize,
    Escape = SDLKey_SDLK_ESCAPE as isize,
    Space = SDLKey_SDLK_SPACE as isize,
    Exclaim = SDLKey_SDLK_EXCLAIM as isize,
    QuoteDbl = SDLKey_SDLK_QUOTEDBL as isize,
    Hash = SDLKey_SDLK_HASH as isize,
    Dollar = SDLKey_SDLK_DOLLAR as isize,
    Ampersand = SDLKey_SDLK_AMPERSAND as isize,
    Quote = SDLKey_SDLK_QUOTE as isize,
    LeftParen = SDLKey_SDLK_LEFTPAREN as isize,
    RightParen = SDLKey_SDLK_RIGHTPAREN as isize,
    Asterisk = SDLKey_SDLK_ASTERISK as isize,
    Plus = SDLKey_SDLK_PLUS as isize,
    Comma = SDLKey_SDLK_COMMA as isize,
    Minus = SDLKey_SDLK_MINUS as isize,
    Period = SDLKey_SDLK_PERIOD as isize,
    Slash = SDLKey_SDLK_SLASH as isize,
    Num0 = SDLKey_SDLK_0 as isize,
    Num1 = SDLKey_SDLK_1 as isize,
    Num2 = SDLKey_SDLK_2 as isize,
    Num3 = SDLKey_SDLK_3 as isize,
    Num4 = SDLKey_SDLK_4 as isize,
    Num5 = SDLKey_SDLK_5 as isize,
    Num6 = SDLKey_SDLK_6 as isize,
    Num7 = SDLKey_SDLK_7 as isize,
    Num8 = SDLKey_SDLK_8 as isize,
    Num9 = SDLKey_SDLK_9 as isize,
    Colon = SDLKey_SDLK_COLON as isize,
    Semicolon = SDLKey_SDLK_SEMICOLON as isize,
    Less = SDLKey_SDLK_LESS as isize,
    Equals = SDLKey_SDLK_EQUALS as isize,
    Greater = SDLKey_SDLK_GREATER as isize,
    Question = SDLKey_SDLK_QUESTION as isize,
    At = SDLKey_SDLK_AT as isize,
    LeftBracket = SDLKey_SDLK_LEFTBRACKET as isize,
    Backslash = SDLKey_SDLK_BACKSLASH as isize,
    RightBracket = SDLKey_SDLK_RIGHTBRACKET as isize,
    Caret = SDLKey_SDLK_CARET as isize,
    Underscore = SDLKey_SDLK_UNDERSCORE as isize,
    Backquote = SDLKey_SDLK_BACKQUOTE as isize,
    A = SDLKey_SDLK_a as isize,
    B = SDLKey_SDLK_b as isize,
    C = SDLKey_SDLK_c as isize,
    D = SDLKey_SDLK_d as isize,
    E = SDLKey_SDLK_e as isize,
    F = SDLKey_SDLK_f as isize,
    G = SDLKey_SDLK_g as isize,
    H = SDLKey_SDLK_h as isize,
    I = SDLKey_SDLK_i as isize,
    J = SDLKey_SDLK_j as isize,
    K = SDLKey_SDLK_k as isize,
    L = SDLKey_SDLK_l as isize,
    M = SDLKey_SDLK_m as isize,
    N = SDLKey_SDLK_n as isize,
    O = SDLKey_SDLK_o as isize,
    P = SDLKey_SDLK_p as isize,
    Q = SDLKey_SDLK_q as isize,
    R = SDLKey_SDLK_r as isize,
    S = SDLKey_SDLK_s as isize,
    T = SDLKey_SDLK_t as isize,
    U = SDLKey_SDLK_u as isize,
    V = SDLKey_SDLK_v as isize,
    W = SDLKey_SDLK_w as isize,
    X = SDLKey_SDLK_x as isize,
    Y = SDLKey_SDLK_y as isize,
    Z = SDLKey_SDLK_z as isize,
    Delete = SDLKey_SDLK_DELETE as isize,
    World0 = SDLKey_SDLK_WORLD_0 as isize,
    World1 = SDLKey_SDLK_WORLD_1 as isize,
    World2 = SDLKey_SDLK_WORLD_2 as isize,
    World3 = SDLKey_SDLK_WORLD_3 as isize,
    World4 = SDLKey_SDLK_WORLD_4 as isize,
    World5 = SDLKey_SDLK_WORLD_5 as isize,
    World6 = SDLKey_SDLK_WORLD_6 as isize,
    World7 = SDLKey_SDLK_WORLD_7 as isize,
    World8 = SDLKey_SDLK_WORLD_8 as isize,
    World9 = SDLKey_SDLK_WORLD_9 as isize,
    World10 = SDLKey_SDLK_WORLD_10 as isize,
    World11 = SDLKey_SDLK_WORLD_11 as isize,
    World12 = SDLKey_SDLK_WORLD_12 as isize,
    World13 = SDLKey_SDLK_WORLD_13 as isize,
    World14 = SDLKey_SDLK_WORLD_14 as isize,
    World15 = SDLKey_SDLK_WORLD_15 as isize,
    World16 = SDLKey_SDLK_WORLD_16 as isize,
    World17 = SDLKey_SDLK_WORLD_17 as isize,
    World18 = SDLKey_SDLK_WORLD_18 as isize,
    World19 = SDLKey_SDLK_WORLD_19 as isize,
    World20 = SDLKey_SDLK_WORLD_20 as isize,
    World21 = SDLKey_SDLK_WORLD_21 as isize,
    World22 = SDLKey_SDLK_WORLD_22 as isize,
    World23 = SDLKey_SDLK_WORLD_23 as isize,
    World24 = SDLKey_SDLK_WORLD_24 as isize,
    World25 = SDLKey_SDLK_WORLD_25 as isize,
    World26 = SDLKey_SDLK_WORLD_26 as isize,
    World27 = SDLKey_SDLK_WORLD_27 as isize,
    World28 = SDLKey_SDLK_WORLD_28 as isize,
    World29 = SDLKey_SDLK_WORLD_29 as isize,
    World30 = SDLKey_SDLK_WORLD_30 as isize,
    World31 = SDLKey_SDLK_WORLD_31 as isize,
    World32 = SDLKey_SDLK_WORLD_32 as isize,
    World33 = SDLKey_SDLK_WORLD_33 as isize,
    World34 = SDLKey_SDLK_WORLD_34 as isize,
    World35 = SDLKey_SDLK_WORLD_35 as isize,
    World36 = SDLKey_SDLK_WORLD_36 as isize,
    World37 = SDLKey_SDLK_WORLD_37 as isize,
    World38 = SDLKey_SDLK_WORLD_38 as isize,
    World39 = SDLKey_SDLK_WORLD_39 as isize,
    World40 = SDLKey_SDLK_WORLD_40 as isize,
    World41 = SDLKey_SDLK_WORLD_41 as isize,
    World42 = SDLKey_SDLK_WORLD_42 as isize,
    World43 = SDLKey_SDLK_WORLD_43 as isize,
    World44 = SDLKey_SDLK_WORLD_44 as isize,
    World45 = SDLKey_SDLK_WORLD_45 as isize,
    World46 = SDLKey_SDLK_WORLD_46 as isize,
    World47 = SDLKey_SDLK_WORLD_47 as isize,
    World48 = SDLKey_SDLK_WORLD_48 as isize,
    World49 = SDLKey_SDLK_WORLD_49 as isize,
    World50 = SDLKey_SDLK_WORLD_50 as isize,
    World51 = SDLKey_SDLK_WORLD_51 as isize,
    World52 = SDLKey_SDLK_WORLD_52 as isize,
    World53 = SDLKey_SDLK_WORLD_53 as isize,
    World54 = SDLKey_SDLK_WORLD_54 as isize,
    World55 = SDLKey_SDLK_WORLD_55 as isize,
    World56 = SDLKey_SDLK_WORLD_56 as isize,
    World57 = SDLKey_SDLK_WORLD_57 as isize,
    World58 = SDLKey_SDLK_WORLD_58 as isize,
    World59 = SDLKey_SDLK_WORLD_59 as isize,
    World60 = SDLKey_SDLK_WORLD_60 as isize,
    World61 = SDLKey_SDLK_WORLD_61 as isize,
    World62 = SDLKey_SDLK_WORLD_62 as isize,
    World63 = SDLKey_SDLK_WORLD_63 as isize,
    World64 = SDLKey_SDLK_WORLD_64 as isize,
    World65 = SDLKey_SDLK_WORLD_65 as isize,
    World66 = SDLKey_SDLK_WORLD_66 as isize,
    World67 = SDLKey_SDLK_WORLD_67 as isize,
    World68 = SDLKey_SDLK_WORLD_68 as isize,
    World69 = SDLKey_SDLK_WORLD_69 as isize,
    World70 = SDLKey_SDLK_WORLD_70 as isize,
    World71 = SDLKey_SDLK_WORLD_71 as isize,
    World72 = SDLKey_SDLK_WORLD_72 as isize,
    World73 = SDLKey_SDLK_WORLD_73 as isize,
    World74 = SDLKey_SDLK_WORLD_74 as isize,
    World75 = SDLKey_SDLK_WORLD_75 as isize,
    World76 = SDLKey_SDLK_WORLD_76 as isize,
    World77 = SDLKey_SDLK_WORLD_77 as isize,
    World78 = SDLKey_SDLK_WORLD_78 as isize,
    World79 = SDLKey_SDLK_WORLD_79 as isize,
    World80 = SDLKey_SDLK_WORLD_80 as isize,
    World81 = SDLKey_SDLK_WORLD_81 as isize,
    World82 = SDLKey_SDLK_WORLD_82 as isize,
    World83 = SDLKey_SDLK_WORLD_83 as isize,
    World84 = SDLKey_SDLK_WORLD_84 as isize,
    World85 = SDLKey_SDLK_WORLD_85 as isize,
    World86 = SDLKey_SDLK_WORLD_86 as isize,
    World87 = SDLKey_SDLK_WORLD_87 as isize,
    World88 = SDLKey_SDLK_WORLD_88 as isize,
    World89 = SDLKey_SDLK_WORLD_89 as isize,
    World90 = SDLKey_SDLK_WORLD_90 as isize,
    World91 = SDLKey_SDLK_WORLD_91 as isize,
    World92 = SDLKey_SDLK_WORLD_92 as isize,
    World93 = SDLKey_SDLK_WORLD_93 as isize,
    World94 = SDLKey_SDLK_WORLD_94 as isize,
    World95 = SDLKey_SDLK_WORLD_95 as isize,
    KeyPad0 = SDLKey_SDLK_KP0 as isize,
    KeyPad1 = SDLKey_SDLK_KP1 as isize,
    KeyPad2 = SDLKey_SDLK_KP2 as isize,
    KeyPad3 = SDLKey_SDLK_KP3 as isize,
    KeyPad4 = SDLKey_SDLK_KP4 as isize,
    KeyPad5 = SDLKey_SDLK_KP5 as isize,
    KeyPad6 = SDLKey_SDLK_KP6 as isize,
    KeyPad7 = SDLKey_SDLK_KP7 as isize,
    KeyPad8 = SDLKey_SDLK_KP8 as isize,
    KeyPad9 = SDLKey_SDLK_KP9 as isize,
    KeyPadPeriod = SDLKey_SDLK_KP_PERIOD as isize,
    KeyPadDivide = SDLKey_SDLK_KP_DIVIDE as isize,
    KeyPadMultiply = SDLKey_SDLK_KP_MULTIPLY as isize,
    KeyPadMinus = SDLKey_SDLK_KP_MINUS as isize,
    KeyPadPlus = SDLKey_SDLK_KP_PLUS as isize,
    KeyPadEnter = SDLKey_SDLK_KP_ENTER as isize,
    KeyPadEquals = SDLKey_SDLK_KP_EQUALS as isize,
    Up = SDLKey_SDLK_UP as isize,
    Down = SDLKey_SDLK_DOWN as isize,
    Right = SDLKey_SDLK_RIGHT as isize,
    Left = SDLKey_SDLK_LEFT as isize,
    Insert = SDLKey_SDLK_INSERT as isize,
    Home = SDLKey_SDLK_HOME as isize,
    End = SDLKey_SDLK_END as isize,
    Pageup = SDLKey_SDLK_PAGEUP as isize,
    Pagedown = SDLKey_SDLK_PAGEDOWN as isize,
    F1 = SDLKey_SDLK_F1 as isize,
    F2 = SDLKey_SDLK_F2 as isize,
    F3 = SDLKey_SDLK_F3 as isize,
    F4 = SDLKey_SDLK_F4 as isize,
    F5 = SDLKey_SDLK_F5 as isize,
    F6 = SDLKey_SDLK_F6 as isize,
    F7 = SDLKey_SDLK_F7 as isize,
    F8 = SDLKey_SDLK_F8 as isize,
    F9 = SDLKey_SDLK_F9 as isize,
    F10 = SDLKey_SDLK_F10 as isize,
    F11 = SDLKey_SDLK_F11 as isize,
    F12 = SDLKey_SDLK_F12 as isize,
    F13 = SDLKey_SDLK_F13 as isize,
    F14 = SDLKey_SDLK_F14 as isize,
    F15 = SDLKey_SDLK_F15 as isize,
    Numlock = SDLKey_SDLK_NUMLOCK as isize,
    Capslock = SDLKey_SDLK_CAPSLOCK as isize,
    Scrollock = SDLKey_SDLK_SCROLLOCK as isize,
    RightShift = SDLKey_SDLK_RSHIFT as isize,
    LeftShift = SDLKey_SDLK_LSHIFT as isize,
    RightCtrl = SDLKey_SDLK_RCTRL as isize,
    LeftCtrl = SDLKey_SDLK_LCTRL as isize,
    RightAlt = SDLKey_SDLK_RALT as isize,
    LeftAlt = SDLKey_SDLK_LALT as isize,
    RightMeta = SDLKey_SDLK_RMETA as isize,
    LeftMeta = SDLKey_SDLK_LMETA as isize,
    LeftSuper = SDLKey_SDLK_LSUPER as isize,
    RightSuper = SDLKey_SDLK_RSUPER as isize,
    Mode = SDLKey_SDLK_MODE as isize,
    Compose = SDLKey_SDLK_COMPOSE as isize,
    Help = SDLKey_SDLK_HELP as isize,
    Print = SDLKey_SDLK_PRINT as isize,
    Sysreq = SDLKey_SDLK_SYSREQ as isize,
    Break = SDLKey_SDLK_BREAK as isize,
    Menu = SDLKey_SDLK_MENU as isize,
    Power = SDLKey_SDLK_POWER as isize,
    Euro = SDLKey_SDLK_EURO as isize,
    Undo = SDLKey_SDLK_UNDO as isize,
}

impl Key {
    fn from_raw(key: u32) -> Result<Self, InvalidKey> {
        use Key::*;
        Ok(match key {
            sdl_sys::SDLKey_SDLK_FIRST => First,
            sdl_sys::SDLKey_SDLK_BACKSPACE => Backspace,
            sdl_sys::SDLKey_SDLK_TAB => Tab,
            sdl_sys::SDLKey_SDLK_CLEAR => Clear,
            sdl_sys::SDLKey_SDLK_RETURN => Return,
            sdl_sys::SDLKey_SDLK_PAUSE => Pause,
            sdl_sys::SDLKey_SDLK_ESCAPE => Escape,
            sdl_sys::SDLKey_SDLK_SPACE => Space,
            sdl_sys::SDLKey_SDLK_EXCLAIM => Exclaim,
            sdl_sys::SDLKey_SDLK_QUOTEDBL => QuoteDbl,
            sdl_sys::SDLKey_SDLK_HASH => Hash,
            sdl_sys::SDLKey_SDLK_DOLLAR => Dollar,
            sdl_sys::SDLKey_SDLK_AMPERSAND => Ampersand,
            sdl_sys::SDLKey_SDLK_QUOTE => Quote,
            sdl_sys::SDLKey_SDLK_LEFTPAREN => LeftParen,
            sdl_sys::SDLKey_SDLK_RIGHTPAREN => RightParen,
            sdl_sys::SDLKey_SDLK_ASTERISK => Asterisk,
            sdl_sys::SDLKey_SDLK_PLUS => Plus,
            sdl_sys::SDLKey_SDLK_COMMA => Comma,
            sdl_sys::SDLKey_SDLK_MINUS => Minus,
            sdl_sys::SDLKey_SDLK_PERIOD => Period,
            sdl_sys::SDLKey_SDLK_SLASH => Slash,
            sdl_sys::SDLKey_SDLK_0 => Num0,
            sdl_sys::SDLKey_SDLK_1 => Num1,
            sdl_sys::SDLKey_SDLK_2 => Num2,
            sdl_sys::SDLKey_SDLK_3 => Num3,
            sdl_sys::SDLKey_SDLK_4 => Num4,
            sdl_sys::SDLKey_SDLK_5 => Num5,
            sdl_sys::SDLKey_SDLK_6 => Num6,
            sdl_sys::SDLKey_SDLK_7 => Num7,
            sdl_sys::SDLKey_SDLK_8 => Num8,
            sdl_sys::SDLKey_SDLK_9 => Num9,
            sdl_sys::SDLKey_SDLK_COLON => Colon,
            sdl_sys::SDLKey_SDLK_SEMICOLON => Semicolon,
            sdl_sys::SDLKey_SDLK_LESS => Less,
            sdl_sys::SDLKey_SDLK_EQUALS => Equals,
            sdl_sys::SDLKey_SDLK_GREATER => Greater,
            sdl_sys::SDLKey_SDLK_QUESTION => Question,
            sdl_sys::SDLKey_SDLK_AT => At,
            sdl_sys::SDLKey_SDLK_LEFTBRACKET => LeftBracket,
            sdl_sys::SDLKey_SDLK_BACKSLASH => Backslash,
            sdl_sys::SDLKey_SDLK_RIGHTBRACKET => RightBracket,
            sdl_sys::SDLKey_SDLK_CARET => Caret,
            sdl_sys::SDLKey_SDLK_UNDERSCORE => Underscore,
            sdl_sys::SDLKey_SDLK_BACKQUOTE => Backquote,
            sdl_sys::SDLKey_SDLK_a => A,
            sdl_sys::SDLKey_SDLK_b => B,
            sdl_sys::SDLKey_SDLK_c => C,
            sdl_sys::SDLKey_SDLK_d => D,
            sdl_sys::SDLKey_SDLK_e => E,
            sdl_sys::SDLKey_SDLK_f => F,
            sdl_sys::SDLKey_SDLK_g => G,
            sdl_sys::SDLKey_SDLK_h => H,
            sdl_sys::SDLKey_SDLK_i => I,
            sdl_sys::SDLKey_SDLK_j => J,
            sdl_sys::SDLKey_SDLK_k => K,
            sdl_sys::SDLKey_SDLK_l => L,
            sdl_sys::SDLKey_SDLK_m => M,
            sdl_sys::SDLKey_SDLK_n => N,
            sdl_sys::SDLKey_SDLK_o => O,
            sdl_sys::SDLKey_SDLK_p => P,
            sdl_sys::SDLKey_SDLK_q => Q,
            sdl_sys::SDLKey_SDLK_r => R,
            sdl_sys::SDLKey_SDLK_s => S,
            sdl_sys::SDLKey_SDLK_t => T,
            sdl_sys::SDLKey_SDLK_u => U,
            sdl_sys::SDLKey_SDLK_v => V,
            sdl_sys::SDLKey_SDLK_w => W,
            sdl_sys::SDLKey_SDLK_x => X,
            sdl_sys::SDLKey_SDLK_y => Y,
            sdl_sys::SDLKey_SDLK_z => Z,
            sdl_sys::SDLKey_SDLK_DELETE => Delete,
            sdl_sys::SDLKey_SDLK_WORLD_0 => World0,
            sdl_sys::SDLKey_SDLK_WORLD_1 => World1,
            sdl_sys::SDLKey_SDLK_WORLD_2 => World2,
            sdl_sys::SDLKey_SDLK_WORLD_3 => World3,
            sdl_sys::SDLKey_SDLK_WORLD_4 => World4,
            sdl_sys::SDLKey_SDLK_WORLD_5 => World5,
            sdl_sys::SDLKey_SDLK_WORLD_6 => World6,
            sdl_sys::SDLKey_SDLK_WORLD_7 => World7,
            sdl_sys::SDLKey_SDLK_WORLD_8 => World8,
            sdl_sys::SDLKey_SDLK_WORLD_9 => World9,
            sdl_sys::SDLKey_SDLK_WORLD_10 => World10,
            sdl_sys::SDLKey_SDLK_WORLD_11 => World11,
            sdl_sys::SDLKey_SDLK_WORLD_12 => World12,
            sdl_sys::SDLKey_SDLK_WORLD_13 => World13,
            sdl_sys::SDLKey_SDLK_WORLD_14 => World14,
            sdl_sys::SDLKey_SDLK_WORLD_15 => World15,
            sdl_sys::SDLKey_SDLK_WORLD_16 => World16,
            sdl_sys::SDLKey_SDLK_WORLD_17 => World17,
            sdl_sys::SDLKey_SDLK_WORLD_18 => World18,
            sdl_sys::SDLKey_SDLK_WORLD_19 => World19,
            sdl_sys::SDLKey_SDLK_WORLD_20 => World20,
            sdl_sys::SDLKey_SDLK_WORLD_21 => World21,
            sdl_sys::SDLKey_SDLK_WORLD_22 => World22,
            sdl_sys::SDLKey_SDLK_WORLD_23 => World23,
            sdl_sys::SDLKey_SDLK_WORLD_24 => World24,
            sdl_sys::SDLKey_SDLK_WORLD_25 => World25,
            sdl_sys::SDLKey_SDLK_WORLD_26 => World26,
            sdl_sys::SDLKey_SDLK_WORLD_27 => World27,
            sdl_sys::SDLKey_SDLK_WORLD_28 => World28,
            sdl_sys::SDLKey_SDLK_WORLD_29 => World29,
            sdl_sys::SDLKey_SDLK_WORLD_30 => World30,
            sdl_sys::SDLKey_SDLK_WORLD_31 => World31,
            sdl_sys::SDLKey_SDLK_WORLD_32 => World32,
            sdl_sys::SDLKey_SDLK_WORLD_33 => World33,
            sdl_sys::SDLKey_SDLK_WORLD_34 => World34,
            sdl_sys::SDLKey_SDLK_WORLD_35 => World35,
            sdl_sys::SDLKey_SDLK_WORLD_36 => World36,
            sdl_sys::SDLKey_SDLK_WORLD_37 => World37,
            sdl_sys::SDLKey_SDLK_WORLD_38 => World38,
            sdl_sys::SDLKey_SDLK_WORLD_39 => World39,
            sdl_sys::SDLKey_SDLK_WORLD_40 => World40,
            sdl_sys::SDLKey_SDLK_WORLD_41 => World41,
            sdl_sys::SDLKey_SDLK_WORLD_42 => World42,
            sdl_sys::SDLKey_SDLK_WORLD_43 => World43,
            sdl_sys::SDLKey_SDLK_WORLD_44 => World44,
            sdl_sys::SDLKey_SDLK_WORLD_45 => World45,
            sdl_sys::SDLKey_SDLK_WORLD_46 => World46,
            sdl_sys::SDLKey_SDLK_WORLD_47 => World47,
            sdl_sys::SDLKey_SDLK_WORLD_48 => World48,
            sdl_sys::SDLKey_SDLK_WORLD_49 => World49,
            sdl_sys::SDLKey_SDLK_WORLD_50 => World50,
            sdl_sys::SDLKey_SDLK_WORLD_51 => World51,
            sdl_sys::SDLKey_SDLK_WORLD_52 => World52,
            sdl_sys::SDLKey_SDLK_WORLD_53 => World53,
            sdl_sys::SDLKey_SDLK_WORLD_54 => World54,
            sdl_sys::SDLKey_SDLK_WORLD_55 => World55,
            sdl_sys::SDLKey_SDLK_WORLD_56 => World56,
            sdl_sys::SDLKey_SDLK_WORLD_57 => World57,
            sdl_sys::SDLKey_SDLK_WORLD_58 => World58,
            sdl_sys::SDLKey_SDLK_WORLD_59 => World59,
            sdl_sys::SDLKey_SDLK_WORLD_60 => World60,
            sdl_sys::SDLKey_SDLK_WORLD_61 => World61,
            sdl_sys::SDLKey_SDLK_WORLD_62 => World62,
            sdl_sys::SDLKey_SDLK_WORLD_63 => World63,
            sdl_sys::SDLKey_SDLK_WORLD_64 => World64,
            sdl_sys::SDLKey_SDLK_WORLD_65 => World65,
            sdl_sys::SDLKey_SDLK_WORLD_66 => World66,
            sdl_sys::SDLKey_SDLK_WORLD_67 => World67,
            sdl_sys::SDLKey_SDLK_WORLD_68 => World68,
            sdl_sys::SDLKey_SDLK_WORLD_69 => World69,
            sdl_sys::SDLKey_SDLK_WORLD_70 => World70,
            sdl_sys::SDLKey_SDLK_WORLD_71 => World71,
            sdl_sys::SDLKey_SDLK_WORLD_72 => World72,
            sdl_sys::SDLKey_SDLK_WORLD_73 => World73,
            sdl_sys::SDLKey_SDLK_WORLD_74 => World74,
            sdl_sys::SDLKey_SDLK_WORLD_75 => World75,
            sdl_sys::SDLKey_SDLK_WORLD_76 => World76,
            sdl_sys::SDLKey_SDLK_WORLD_77 => World77,
            sdl_sys::SDLKey_SDLK_WORLD_78 => World78,
            sdl_sys::SDLKey_SDLK_WORLD_79 => World79,
            sdl_sys::SDLKey_SDLK_WORLD_80 => World80,
            sdl_sys::SDLKey_SDLK_WORLD_81 => World81,
            sdl_sys::SDLKey_SDLK_WORLD_82 => World82,
            sdl_sys::SDLKey_SDLK_WORLD_83 => World83,
            sdl_sys::SDLKey_SDLK_WORLD_84 => World84,
            sdl_sys::SDLKey_SDLK_WORLD_85 => World85,
            sdl_sys::SDLKey_SDLK_WORLD_86 => World86,
            sdl_sys::SDLKey_SDLK_WORLD_87 => World87,
            sdl_sys::SDLKey_SDLK_WORLD_88 => World88,
            sdl_sys::SDLKey_SDLK_WORLD_89 => World89,
            sdl_sys::SDLKey_SDLK_WORLD_90 => World90,
            sdl_sys::SDLKey_SDLK_WORLD_91 => World91,
            sdl_sys::SDLKey_SDLK_WORLD_92 => World92,
            sdl_sys::SDLKey_SDLK_WORLD_93 => World93,
            sdl_sys::SDLKey_SDLK_WORLD_94 => World94,
            sdl_sys::SDLKey_SDLK_WORLD_95 => World95,
            sdl_sys::SDLKey_SDLK_KP0 => KeyPad0,
            sdl_sys::SDLKey_SDLK_KP1 => KeyPad1,
            sdl_sys::SDLKey_SDLK_KP2 => KeyPad2,
            sdl_sys::SDLKey_SDLK_KP3 => KeyPad3,
            sdl_sys::SDLKey_SDLK_KP4 => KeyPad4,
            sdl_sys::SDLKey_SDLK_KP5 => KeyPad5,
            sdl_sys::SDLKey_SDLK_KP6 => KeyPad6,
            sdl_sys::SDLKey_SDLK_KP7 => KeyPad7,
            sdl_sys::SDLKey_SDLK_KP8 => KeyPad8,
            sdl_sys::SDLKey_SDLK_KP9 => KeyPad9,
            sdl_sys::SDLKey_SDLK_KP_PERIOD => KeyPadPeriod,
            sdl_sys::SDLKey_SDLK_KP_DIVIDE => KeyPadDivide,
            sdl_sys::SDLKey_SDLK_KP_MULTIPLY => KeyPadMultiply,
            sdl_sys::SDLKey_SDLK_KP_MINUS => KeyPadMinus,
            sdl_sys::SDLKey_SDLK_KP_PLUS => KeyPadPlus,
            sdl_sys::SDLKey_SDLK_KP_ENTER => KeyPadEnter,
            sdl_sys::SDLKey_SDLK_KP_EQUALS => KeyPadEquals,
            sdl_sys::SDLKey_SDLK_UP => Up,
            sdl_sys::SDLKey_SDLK_DOWN => Down,
            sdl_sys::SDLKey_SDLK_RIGHT => Right,
            sdl_sys::SDLKey_SDLK_LEFT => Left,
            sdl_sys::SDLKey_SDLK_INSERT => Insert,
            sdl_sys::SDLKey_SDLK_HOME => Home,
            sdl_sys::SDLKey_SDLK_END => End,
            sdl_sys::SDLKey_SDLK_PAGEUP => Pageup,
            sdl_sys::SDLKey_SDLK_PAGEDOWN => Pagedown,
            sdl_sys::SDLKey_SDLK_F1 => F1,
            sdl_sys::SDLKey_SDLK_F2 => F2,
            sdl_sys::SDLKey_SDLK_F3 => F3,
            sdl_sys::SDLKey_SDLK_F4 => F4,
            sdl_sys::SDLKey_SDLK_F5 => F5,
            sdl_sys::SDLKey_SDLK_F6 => F6,
            sdl_sys::SDLKey_SDLK_F7 => F7,
            sdl_sys::SDLKey_SDLK_F8 => F8,
            sdl_sys::SDLKey_SDLK_F9 => F9,
            sdl_sys::SDLKey_SDLK_F10 => F10,
            sdl_sys::SDLKey_SDLK_F11 => F11,
            sdl_sys::SDLKey_SDLK_F12 => F12,
            sdl_sys::SDLKey_SDLK_F13 => F13,
            sdl_sys::SDLKey_SDLK_F14 => F14,
            sdl_sys::SDLKey_SDLK_F15 => F15,
            sdl_sys::SDLKey_SDLK_NUMLOCK => Numlock,
            sdl_sys::SDLKey_SDLK_CAPSLOCK => Capslock,
            sdl_sys::SDLKey_SDLK_SCROLLOCK => Scrollock,
            sdl_sys::SDLKey_SDLK_RSHIFT => RightShift,
            sdl_sys::SDLKey_SDLK_LSHIFT => LeftShift,
            sdl_sys::SDLKey_SDLK_RCTRL => RightCtrl,
            sdl_sys::SDLKey_SDLK_LCTRL => LeftCtrl,
            sdl_sys::SDLKey_SDLK_RALT => RightAlt,
            sdl_sys::SDLKey_SDLK_LALT => LeftAlt,
            sdl_sys::SDLKey_SDLK_RMETA => RightMeta,
            sdl_sys::SDLKey_SDLK_LMETA => LeftMeta,
            sdl_sys::SDLKey_SDLK_LSUPER => LeftSuper,
            sdl_sys::SDLKey_SDLK_RSUPER => RightSuper,
            sdl_sys::SDLKey_SDLK_MODE => Mode,
            sdl_sys::SDLKey_SDLK_COMPOSE => Compose,
            sdl_sys::SDLKey_SDLK_HELP => Help,
            sdl_sys::SDLKey_SDLK_PRINT => Print,
            sdl_sys::SDLKey_SDLK_SYSREQ => Sysreq,
            sdl_sys::SDLKey_SDLK_BREAK => Break,
            sdl_sys::SDLKey_SDLK_MENU => Menu,
            sdl_sys::SDLKey_SDLK_POWER => Power,
            sdl_sys::SDLKey_SDLK_EURO => Euro,
            sdl_sys::SDLKey_SDLK_UNDO => Undo,
            _ => return Err(InvalidKey),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid virtual key")]
pub struct InvalidKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mod {
    None = SDLMod_KMOD_NONE as isize,
    LeftShift = SDLMod_KMOD_LSHIFT as isize,
    RightShift = SDLMod_KMOD_RSHIFT as isize,
    LeftCtrl = SDLMod_KMOD_LCTRL as isize,
    RightCtrl = SDLMod_KMOD_RCTRL as isize,
    LeftAlt = SDLMod_KMOD_LALT as isize,
    RightAlt = SDLMod_KMOD_RALT as isize,
    LeftMeta = SDLMod_KMOD_LMETA as isize,
    RightMeta = SDLMod_KMOD_RMETA as isize,
    Num = SDLMod_KMOD_NUM as isize,
    Caps = SDLMod_KMOD_CAPS as isize,
    Mode = SDLMod_KMOD_MODE as isize,
}

impl Mod {
    fn from_raw(raw: u32) -> Result<Self, InvalidMod> {
        use Mod::*;
        Ok(match raw {
            sdl_sys::SDLMod_KMOD_NONE => None,
            sdl_sys::SDLMod_KMOD_LSHIFT => LeftShift,
            sdl_sys::SDLMod_KMOD_RSHIFT => RightShift,
            sdl_sys::SDLMod_KMOD_LCTRL => LeftCtrl,
            sdl_sys::SDLMod_KMOD_RCTRL => RightCtrl,
            sdl_sys::SDLMod_KMOD_LALT => LeftAlt,
            sdl_sys::SDLMod_KMOD_RALT => RightAlt,
            sdl_sys::SDLMod_KMOD_LMETA => LeftMeta,
            sdl_sys::SDLMod_KMOD_RMETA => RightMeta,
            sdl_sys::SDLMod_KMOD_NUM => Num,
            sdl_sys::SDLMod_KMOD_CAPS => Caps,
            sdl_sys::SDLMod_KMOD_MODE => Mode,
            _ => return Err(InvalidMod),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid key modifier")]
pub struct InvalidMod;