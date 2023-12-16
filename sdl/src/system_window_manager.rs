use crate::Version;

use sdl_sys::SDL_SysWMmsg;

#[cfg(sdl_video_driver_x11)]
mod x11 {
    use core::fmt;
    use std::{
        ops::Deref,
        os::raw::{c_char, c_long, c_short},
    };

    use sdl_sys::{
        SDL_SysWMmsg__bindgen_ty_1, XButtonEvent, XCirculateEvent, XCirculateRequestEvent,
        XClientMessageEvent, XColormapEvent, XConfigureEvent, XConfigureRequestEvent,
        XCreateWindowEvent, XCrossingEvent, XDestroyWindowEvent, XExposeEvent, XFocusChangeEvent,
        XGenericEvent, XGraphicsExposeEvent, XGravityEvent, XKeyEvent, XKeymapEvent, XMapEvent,
        XMapRequestEvent, XMappingEvent, XMotionEvent, XNoExposeEvent, XPropertyEvent,
        XReparentEvent, XResizeRequestEvent, XSelectionClearEvent, XSelectionEvent,
        XSelectionRequestEvent, XUnmapEvent, XVisibilityEvent,
    };

    use crate::convert;

    use super::{SDL_SysWMmsg, Version};

    #[derive(Clone)]
    pub struct XEvent(pub sdl_sys::XEvent);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Message {
        pub version: Version,
        pub subsystem: Subsystem,
        pub event: XEvent,
    }

    impl Message {
        #[must_use]
        pub fn to_raw(&self) -> SDL_SysWMmsg {
            let &Self {
                version,
                subsystem,
                ref event,
            } = self;
            let version = version.to_raw();
            let subsystem = subsystem as u32;

            SDL_SysWMmsg {
                version,
                subsystem,
                event: SDL_SysWMmsg__bindgen_ty_1 { xevent: event.0 },
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Subsystem {
        X11 = convert::u32_to_isize(sdl_sys::SDL_SYSWM_TYPE_SDL_SYSWM_X11),
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    #[error("invalid SDL system window manager message")]
    pub struct InvalidSystemWindowManagerMessage;

    impl TryFrom<&SDL_SysWMmsg> for Message {
        type Error = InvalidSystemWindowManagerMessage;

        fn try_from(msg: &SDL_SysWMmsg) -> Result<Self, Self::Error> {
            let &SDL_SysWMmsg {
                version,
                subsystem,
                event,
            } = msg;

            let version = version.into();
            let subsystem = match subsystem {
                sdl_sys::SDL_SYSWM_TYPE_SDL_SYSWM_X11 => Subsystem::X11,
                _ => return Err(InvalidSystemWindowManagerMessage),
            };
            let event = XEvent(unsafe { event.xevent });

            Ok(Self {
                version,
                subsystem,
                event,
            })
        }
    }

    impl Deref for XEvent {
        type Target = sdl_sys::XEvent;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl AsRef<sdl_sys::XEvent> for XEvent {
        fn as_ref(&self) -> &sdl_sys::XEvent {
            &self.0
        }
    }

    impl XEvent {
        fn to_event_variant_ref(&self) -> XEventVariantRef<'_> {
            XEventVariantRef::try_from(&self.0).unwrap()
        }
    }

    impl PartialEq for XEvent {
        fn eq(&self, other: &Self) -> bool {
            self.to_event_variant_ref() == other.to_event_variant_ref()
        }
    }

    impl Eq for XEvent {}

    impl fmt::Debug for XEvent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use XEventVariantRef as E;

            let mut dbg = f.debug_struct("XEvent");

            match self.to_event_variant_ref() {
                E::Key(event) => dbg.field("XKeyEvent", event),
                E::Button(event) => dbg.field("XButtonEvent", event),
                E::Motion(event) => dbg.field("XMotionEvent", event),
                E::Crossing(event) => dbg.field("XCrossingEvent", event),
                E::Focus(event) => dbg.field("XFocusEvent", event),
                E::Expose(event) => dbg.field("XExposeEvent", event),
                E::GraphicExpose(event) => dbg.field("XGraphicExposeEvent", event),
                E::NoExpose(event) => dbg.field("XNoExposeEvent", event),
                E::Visibility(event) => dbg.field("XVisibilityEvent", event),
                E::Createwindow(event) => dbg.field("XCreatewindowEvent", event),
                E::DestroyWindow(event) => dbg.field("XDestroyWindowEvent", event),
                E::Unmap(event) => dbg.field("XUnmapEvent", event),
                E::Map(event) => dbg.field("XMapEvent", event),
                E::MapRequest(event) => dbg.field("XMapRequestEvent", event),
                E::Reparent(event) => dbg.field("XReparentEvent", event),
                E::Configure(event) => dbg.field("XConfigureEvent", event),
                E::Gravity(event) => dbg.field("XGravityEvent", event),
                E::ResizeRequest(event) => dbg.field("XResizeRequestEvent", event),
                E::ConfigureRequest(event) => dbg.field("XConfigureRequestEvent", event),
                E::Circulate(event) => dbg.field("XCirculateEvent", event),
                E::CirculateRequest(event) => dbg.field("XCirculateRequestEvent", event),
                E::Property(event) => dbg.field("XPropertyEvent", event),
                E::SelectionClear(event) => dbg.field("XSelectionClearEvent", event),
                E::SelectionRequest(event) => dbg.field("XSelectionRequestEvent", event),
                E::Selection(event) => dbg.field("XSelectionEvent", event),
                E::Colormap(event) => dbg.field("XColormapEvent", event),
                E::Client(event) => dbg.field("XClientEvent", &XClientMessageEventRef(event)),
                E::Mapping(event) => dbg.field("XMappingEvent", event),
                E::Keymap(event) => dbg.field("XKeymapEvent", event),
                E::Generic(event) => dbg.field("XGenericEvent", event),
            };

            dbg.finish()
        }
    }

    #[derive(Clone, Copy)]
    enum XEventVariantRef<'a> {
        Key(&'a XKeyEvent),
        Button(&'a XButtonEvent),
        Motion(&'a XMotionEvent),
        Crossing(&'a XCrossingEvent),
        Focus(&'a XFocusChangeEvent),
        Expose(&'a XExposeEvent),
        GraphicExpose(&'a XGraphicsExposeEvent),
        NoExpose(&'a XNoExposeEvent),
        Visibility(&'a XVisibilityEvent),
        Createwindow(&'a XCreateWindowEvent),
        DestroyWindow(&'a XDestroyWindowEvent),
        Unmap(&'a XUnmapEvent),
        Map(&'a XMapEvent),
        MapRequest(&'a XMapRequestEvent),
        Reparent(&'a XReparentEvent),
        Configure(&'a XConfigureEvent),
        Gravity(&'a XGravityEvent),
        ResizeRequest(&'a XResizeRequestEvent),
        ConfigureRequest(&'a XConfigureRequestEvent),
        Circulate(&'a XCirculateEvent),
        CirculateRequest(&'a XCirculateRequestEvent),
        Property(&'a XPropertyEvent),
        SelectionClear(&'a XSelectionClearEvent),
        SelectionRequest(&'a XSelectionRequestEvent),
        Selection(&'a XSelectionEvent),
        Colormap(&'a XColormapEvent),
        Client(&'a XClientMessageEvent),
        Mapping(&'a XMappingEvent),
        Keymap(&'a XKeymapEvent),
        Generic(&'a XGenericEvent),
    }

    impl<'a> PartialEq for XEventVariantRef<'a> {
        fn eq(&self, other: &Self) -> bool {
            use XEventVariantRef::{
                Button, Circulate, CirculateRequest, Client, Colormap, Configure, ConfigureRequest,
                Createwindow, Crossing, DestroyWindow, Expose, Focus, Generic, GraphicExpose,
                Gravity, Key, Keymap, Map, MapRequest, Mapping, Motion, NoExpose, Property,
                Reparent, ResizeRequest, Selection, SelectionClear, SelectionRequest, Unmap,
                Visibility,
            };
            match (*self, *other) {
                (Key(a), Key(b)) => a.eq(b),
                (Button(a), Button(b)) => a.eq(b),
                (Motion(a), Motion(b)) => a.eq(b),
                (Crossing(a), Crossing(b)) => a.eq(b),
                (Focus(a), Focus(b)) => a.eq(b),
                (Expose(a), Expose(b)) => a.eq(b),
                (GraphicExpose(a), GraphicExpose(b)) => a.eq(b),
                (NoExpose(a), NoExpose(b)) => a.eq(b),
                (Visibility(a), Visibility(b)) => a.eq(b),
                (Createwindow(a), Createwindow(b)) => a.eq(b),
                (DestroyWindow(a), DestroyWindow(b)) => a.eq(b),
                (Unmap(a), Unmap(b)) => a.eq(b),
                (Map(a), Map(b)) => a.eq(b),
                (MapRequest(a), MapRequest(b)) => a.eq(b),
                (Reparent(a), Reparent(b)) => a.eq(b),
                (Configure(a), Configure(b)) => a.eq(b),
                (Gravity(a), Gravity(b)) => a.eq(b),
                (ResizeRequest(a), ResizeRequest(b)) => a.eq(b),
                (ConfigureRequest(a), ConfigureRequest(b)) => a.eq(b),
                (Circulate(a), Circulate(b)) => a.eq(b),
                (CirculateRequest(a), CirculateRequest(b)) => a.eq(b),
                (Property(a), Property(b)) => a.eq(b),
                (SelectionClear(a), SelectionClear(b)) => a.eq(b),
                (SelectionRequest(a), SelectionRequest(b)) => a.eq(b),
                (Selection(a), Selection(b)) => a.eq(b),
                (Colormap(a), Colormap(b)) => a.eq(b),
                (Client(a), Client(b)) => {
                    let a = XClientMessageDataRef::try_from(a);
                    let b = XClientMessageDataRef::try_from(b);
                    match (a, b) {
                        (Ok(a), Ok(b)) => a.eq(&b),
                        _ => false,
                    }
                }
                (Mapping(a), Mapping(b)) => a.eq(b),
                (Keymap(a), Keymap(b)) => a.eq(b),
                (Generic(a), Generic(b)) => a.eq(b),
                _ => false,
            }
        }
    }

    impl Eq for XEventVariantRef<'_> {}

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct InvalidXEvent;

    impl<'a> TryFrom<&'a sdl_sys::XEvent> for XEventVariantRef<'a> {
        type Error = InvalidXEvent;

        fn try_from(event: &'a sdl_sys::XEvent) -> Result<Self, Self::Error> {
            // Taken from X11/X.h
            const KEY_PRESS: i32 = 2;
            const KEY_RELEASE: i32 = 3;
            const BUTTON_PRESS: i32 = 4;
            const BUTTON_RELEASE: i32 = 5;
            const MOTION_NOTIFY: i32 = 6;
            const ENTER_NOTIFY: i32 = 7;
            const LEAVE_NOTIFY: i32 = 8;
            const FOCUS_IN: i32 = 9;
            const FOCUS_OUT: i32 = 10;
            const KEYMAP_NOTIFY: i32 = 11;
            const EXPOSE: i32 = 12;
            const GRAPHICS_EXPOSE: i32 = 13;
            const NO_EXPOSE: i32 = 14;
            const VISIBILITY_NOTIFY: i32 = 15;
            const CREATE_NOTIFY: i32 = 16;
            const DESTROY_NOTIFY: i32 = 17;
            const UNMAP_NOTIFY: i32 = 18;
            const MAP_NOTIFY: i32 = 19;
            const MAP_REQUEST: i32 = 20;
            const REPARENT_NOTIFY: i32 = 21;
            const CONFIGURE_NOTIFY: i32 = 22;
            const CONFIGURE_REQUEST: i32 = 23;
            const GRAVITY_NOTIFY: i32 = 24;
            const RESIZE_REQUEST: i32 = 25;
            const CIRCULATE_NOTIFY: i32 = 26;
            const CIRCULATE_REQUEST: i32 = 27;
            const PROPERTY_NOTIFY: i32 = 28;
            const SELECTION_CLEAR: i32 = 29;
            const SELECTION_REQUEST: i32 = 30;
            const SELECTION_NOTIFY: i32 = 31;
            const COLORMAP_NOTIFY: i32 = 32;
            const CLIENT_MESSAGE: i32 = 33;
            const MAPPING_NOTIFY: i32 = 34;
            const GENERIC_EVENT: i32 = 35;

            use XEventVariantRef::{
                Button, Circulate, CirculateRequest, Client, Colormap, Configure, ConfigureRequest,
                Createwindow, Crossing, DestroyWindow, Expose, Focus, Generic, GraphicExpose,
                Gravity, Key, Keymap, Map, MapRequest, Mapping, Motion, NoExpose, Property,
                Reparent, ResizeRequest, Selection, SelectionClear, SelectionRequest, Unmap,
                Visibility,
            };
            Ok(match unsafe { event.type_ } {
                KEY_PRESS | KEY_RELEASE => Key(unsafe { &event.xkey }),
                BUTTON_PRESS | BUTTON_RELEASE => Button(unsafe { &event.xbutton }),
                MOTION_NOTIFY => Motion(unsafe { &event.xmotion }),
                ENTER_NOTIFY | LEAVE_NOTIFY => Crossing(unsafe { &event.xcrossing }),
                FOCUS_IN | FOCUS_OUT => Focus(unsafe { &event.xfocus }),
                KEYMAP_NOTIFY => Keymap(unsafe { &event.xkeymap }),
                EXPOSE => Expose(unsafe { &event.xexpose }),
                GRAPHICS_EXPOSE => GraphicExpose(unsafe { &event.xgraphicsexpose }),
                NO_EXPOSE => NoExpose(unsafe { &event.xnoexpose }),
                VISIBILITY_NOTIFY => Visibility(unsafe { &event.xvisibility }),
                CREATE_NOTIFY => Createwindow(unsafe { &event.xcreatewindow }),
                DESTROY_NOTIFY => DestroyWindow(unsafe { &event.xdestroywindow }),
                UNMAP_NOTIFY => Unmap(unsafe { &event.xunmap }),
                MAP_NOTIFY => Map(unsafe { &event.xmap }),
                MAP_REQUEST => MapRequest(unsafe { &event.xmaprequest }),
                REPARENT_NOTIFY => Reparent(unsafe { &event.xreparent }),
                CONFIGURE_NOTIFY => Configure(unsafe { &event.xconfigure }),
                CONFIGURE_REQUEST => ConfigureRequest(unsafe { &event.xconfigurerequest }),
                GRAVITY_NOTIFY => Gravity(unsafe { &event.xgravity }),
                RESIZE_REQUEST => ResizeRequest(unsafe { &event.xresizerequest }),
                CIRCULATE_NOTIFY => Circulate(unsafe { &event.xcirculate }),
                CIRCULATE_REQUEST => CirculateRequest(unsafe { &event.xcirculaterequest }),
                PROPERTY_NOTIFY => Property(unsafe { &event.xproperty }),
                SELECTION_CLEAR => SelectionClear(unsafe { &event.xselectionclear }),
                SELECTION_REQUEST => SelectionRequest(unsafe { &event.xselectionrequest }),
                SELECTION_NOTIFY => Selection(unsafe { &event.xselection }),
                COLORMAP_NOTIFY => Colormap(unsafe { &event.xcolormap }),
                CLIENT_MESSAGE => Client(unsafe { &event.xclient }),
                MAPPING_NOTIFY => Mapping(unsafe { &event.xmapping }),
                GENERIC_EVENT => Generic(unsafe { &event.xgeneric }),
                _ => return Err(InvalidXEvent),
            })
        }
    }

    #[derive(Clone, Copy)]
    pub struct XClientMessageEventRef<'a>(&'a XClientMessageEvent);

    impl PartialEq for XClientMessageEventRef<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.0.type_ == other.0.type_
                && self.0.serial == other.0.serial
                && self.0.send_event == other.0.send_event
                && self.0.display == other.0.display
                && self.0.window == other.0.window
                && self.0.format == other.0.format
                && XClientMessageDataRef::try_from(self.0)
                    == XClientMessageDataRef::try_from(other.0)
        }
    }

    impl Eq for XClientMessageEventRef<'_> {}

    impl fmt::Debug for XClientMessageEventRef<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("XClientMessageEvent")
                .field("type_", &self.0.type_)
                .field("serial", &self.0.serial)
                .field("send_event", &self.0.send_event)
                .field("display", &self.0.display)
                .field("window", &self.0.window)
                .field("message_type", &self.0.message_type)
                .field("format", &self.0.format)
                .field("data", &XClientMessageDataRef::try_from(self.0).unwrap())
                .finish()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum XClientMessageDataRef<'a> {
        Bytes(&'a [c_char; 20]),
        Short(&'a [c_short; 10]),
        Long(&'a [c_long; 5]),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InvalidXClientMessageType(u64);

    impl<'a> TryFrom<&'a XClientMessageEvent> for XClientMessageDataRef<'a> {
        type Error = InvalidXClientMessageType;

        fn try_from(event: &'a XClientMessageEvent) -> Result<Self, Self::Error> {
            Ok(match event.message_type {
                8 => Self::Bytes(unsafe { &event.data.b }),
                16 => Self::Short(unsafe { &event.data.s }),
                32 => Self::Long(unsafe { &event.data.l }),
                n => return Err(InvalidXClientMessageType(n)),
            })
        }
    }
}

#[cfg(sdl_video_driver_x11)]
pub use x11::*;

// TODO: sdl_video_driver_nanox
// TODO: sdl_video_driver_windib || sdl_video_driver_ddraw || sdl_video_driver_gapi
// TODO: sdl_video_driver_riscos
// TODO: sdl_video_driver_photon

#[cfg(not(any(sdl_video_driver_x11)))]
mod generic {
    use std::os::raw::c_int;

    use super::*;

    /// The generic custom event structure
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Message {
        pub version: Version,
        pub data: c_int,
    }

    /// The generic custom window manager information structure
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Info {
        pub version: Version,
        pub data: c_int,
    }

    impl From<&SDL_SysWMmsg> for Message {
        fn from(message: &SDL_SysWMmsg) -> Self {
            let &SDL_SysWMmsg { version, data } = message;

            let version = version.into();
            Self { version, data }
        }
    }
}

#[cfg(not(any(sdl_video_driver_x11)))]
pub use generic::*;
