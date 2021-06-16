use crate::{
    b_font::{font_height, print_string_font},
    defs::{
        AssembleCombatWindowFlags, Cmds, MapTile, MAXWAYPOINTS, MAX_WP_CONNECTIONS, NUM_MAP_BLOCKS,
    },
    enemy::shuffle_enemys,
    graphics::{clear_graph_mem, putpixel, NE_SCREEN},
    input::SDL_Delay,
    structs::{Level, Waypoint},
    vars::FULL_USER_RECT,
    view::{fill_rect, BLACK},
    Data, CUR_LEVEL, ME,
};

use cstr::cstr;
use log::{info, warn};
use sdl::{
    event::ll::SDLK_F1,
    event::ll::{
        SDLK_KP0, SDLK_KP1, SDLK_KP2, SDLK_KP3, SDLK_KP4, SDLK_KP5, SDLK_KP6, SDLK_KP7, SDLK_KP8,
        SDLK_KP9, SDLK_KP_PLUS,
    },
    video::ll::{SDL_Flip, SDL_LockSurface, SDL_UnlockSurface},
};
use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::{c_char, c_int},
    ptr::null_mut,
};

const HIGHLIGHTCOLOR: u32 = 255;
const HIGHLIGHTCOLOR2: i32 = 100;

impl Data {
    /// This function is provides the Level Editor integrated into
    /// freedroid.  Actually this function is a submenu of the big
    /// Escape Menu.  In here you can edit the level and upon pressing
    /// escape enter a further submenu where you can save the level,
    /// change level name and quit from level editing.
    pub unsafe fn level_editor(&mut self) {
        let mut done = false;
        let mut origin_waypoint: c_int = -1;

        let keymap_offset = 15;

        let rect = self.vars.user_rect;
        self.vars.user_rect = self.vars.screen_rect; // level editor can use the full screen!
        let mut src_wp = null_mut();

        while done.not() {
            if self.cmd_is_active_r(Cmds::Menu) {
                self.show_level_editor_menu();
                if self.menu.quit_level_editor {
                    done = true;
                    self.global.current_combat_scale_factor = 1.;
                    self.set_combat_scale_to(self.global.current_combat_scale_factor);
                    self.menu.quit_level_editor = false;
                }
                continue;
            }

            let block_x = (ME.pos.x).round() as c_int;
            let block_y = (ME.pos.y).round() as c_int;

            fill_rect(self.vars.user_rect, BLACK);
            self.assemble_combat_picture(AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits().into());
            self.highlight_current_block();
            self.show_waypoints();

            // show line between a selected connection-origin and the current block
            if origin_waypoint != -1 {
                self.draw_line_between_tiles(
                    block_x as f32,
                    block_y as f32,
                    (*CUR_LEVEL).all_waypoints[usize::try_from(origin_waypoint).unwrap()]
                        .x
                        .into(),
                    (*CUR_LEVEL).all_waypoints[usize::try_from(origin_waypoint).unwrap()]
                        .y
                        .into(),
                    HIGHLIGHTCOLOR2,
                );
            }

            print_string_font(
                NE_SCREEN,
                self.global.font0_b_font,
                i32::from(FULL_USER_RECT.x) + i32::from(FULL_USER_RECT.w) / 3,
                i32::from(FULL_USER_RECT.y) + i32::from(FULL_USER_RECT.h)
                    - font_height(&*self.global.font0_b_font),
                format_args!("Press F1 for keymap"),
            );

            SDL_Flip(NE_SCREEN);

            // If the user of the Level editor pressed some cursor keys, move the
            // highlited filed (that is Me.pos) accordingly. This is done here:
            //
            if self.left_pressed_r() && ME.pos.x.round() > 0. {
                ME.pos.x -= 1.;
            }

            if self.right_pressed_r() && (ME.pos.x.round() as c_int) < (*CUR_LEVEL).xlen - 1 {
                ME.pos.x += 1.;
            }

            if self.up_pressed_r() && ME.pos.y.round() > 0. {
                ME.pos.y -= 1.;
            }

            if self.down_pressed_r() && (ME.pos.y.round() as c_int) < (*CUR_LEVEL).ylen - 1 {
                ME.pos.y += 1.;
            }

            if self.key_is_pressed_r(SDLK_F1.try_into().unwrap()) {
                let mut k = 3;
                self.make_grid_on_screen(None);
                self.centered_put_string(
                    NE_SCREEN,
                    k * font_height(&*self.global.menu_b_font),
                    b"Level Editor Keymap",
                );
                k += 2;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"Use cursor keys to move around.",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"Use number pad to plant walls.",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"Use shift and number pad to plant extras.",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"R...Refresh, 1-5...Blocktype 1-5, L...Lift",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"F...Fine grid, T/SHIFT + T...Doors",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"M...Alert, E...Enter tile by number",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"Space/Enter...Floor",
                );
                k += 2;

                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"I/O...zoom INTO/OUT OF the map",
                );
                k += 2;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"P...toggle wayPOINT on/off",
                );
                k += 1;
                self.put_string(
                    NE_SCREEN,
                    keymap_offset,
                    (k) * font_height(&*self.global.menu_b_font),
                    b"C...start/end waypoint CONNECTION",
                );

                SDL_Flip(NE_SCREEN);
                while !self.fire_pressed_r() && !self.escape_pressed_r() && !self.return_pressed_r()
                {
                    SDL_Delay(1);
                }
            }

            //--------------------
            // Since the level editor will not always be able to
            // immediately feature all the the map tiles that might
            // have been added recently, we should offer a feature, so that you can
            // specify the value of a map piece just numerically.  This will be
            // done upon pressing the 'e' key.
            //
            if self.key_is_pressed_r(b'e'.into()) {
                self.centered_put_string(
                    NE_SCREEN,
                    6 * font_height(&*self.global.menu_b_font),
                    b"Please enter new value: ",
                );
                SDL_Flip(NE_SCREEN);
                let numeric_input_string = self.get_string(10, 2);
                let mut special_map_value: c_int = 0;
                libc::sscanf(
                    numeric_input_string,
                    cstr!("%d").as_ptr() as *mut c_char,
                    &mut special_map_value,
                );
                if special_map_value >= NUM_MAP_BLOCKS.try_into().unwrap() {
                    special_map_value = 0;
                }
                *((*CUR_LEVEL).map[usize::try_from(block_y).unwrap()])
                    .add(block_x.try_into().unwrap()) = special_map_value.try_into().unwrap();
            }

            //If the person using the level editor decides he/she wants a different
            //scale for the editing process, he/she may say so by using the O/I keys.
            if self.key_is_pressed_r(b'o'.into()) {
                if self.global.current_combat_scale_factor > 0.25 {
                    self.global.current_combat_scale_factor -= 0.25;
                }
                self.set_combat_scale_to(self.global.current_combat_scale_factor);
            }
            if self.key_is_pressed_r(b'i'.into()) {
                self.global.current_combat_scale_factor += 0.25;
                self.set_combat_scale_to(self.global.current_combat_scale_factor);
            }

            // toggle waypoint on current square.  That means either removed or added.
            // And in case of removal, also the connections must be removed.
            if self.key_is_pressed_r(b'p'.into()) {
                // find out if there is a waypoint on the current square
                let mut i = 0;
                while i < usize::try_from((*CUR_LEVEL).num_waypoints).unwrap() {
                    if i32::from((*CUR_LEVEL).all_waypoints[i].x) == block_x
                        && i32::from((*CUR_LEVEL).all_waypoints[i].y) == block_y
                    {
                        break;
                    }

                    i += 1;
                }

                // if its waypoint already, this waypoint must be deleted.
                if i < usize::try_from((*CUR_LEVEL).num_waypoints).unwrap() {
                    delete_waypoint(&mut *CUR_LEVEL, i.try_into().unwrap());
                } else {
                    // if its not a waypoint already, it must be made into one
                    create_waypoint(&mut *CUR_LEVEL, block_x, block_y);
                }
            } // if 'p' pressed (toggle waypoint)

            // create a connection between waypoints.  If this is the first selected waypoint, its
            // an origin and the second "C"-pressed waypoint will be used a target.
            // If origin and destination are the same, the operation is cancelled.
            if self.key_is_pressed_r(b'c'.into()) {
                // Determine which waypoint is currently targeted
                let mut i = 0;
                while i < usize::try_from((*CUR_LEVEL).num_waypoints).unwrap() {
                    if i32::from((*CUR_LEVEL).all_waypoints[i].x) == block_x
                        && i32::from((*CUR_LEVEL).all_waypoints[i].y) == block_y
                    {
                        break;
                    }

                    i += 1;
                }

                if i == usize::try_from((*CUR_LEVEL).num_waypoints).unwrap() {
                    warn!("Sorry, no waypoint here to connect.");
                } else if origin_waypoint == -1 {
                    origin_waypoint = i.try_into().unwrap();
                    src_wp = &mut (*CUR_LEVEL).all_waypoints[i];
                    if (*src_wp).num_connections < c_int::try_from(MAX_WP_CONNECTIONS).unwrap() {
                        info!("Waypoint nr. {}. selected as origin", i);
                    } else {
                        warn!(
                        "Sorry, maximal number of waypoint-connections ({}) reached! Operation \
                         not possible.",
                        MAX_WP_CONNECTIONS,
                    );
                        origin_waypoint = -1;
                        src_wp = null_mut();
                    }
                } else if origin_waypoint == c_int::try_from(i).unwrap() {
                    info!("Origin==Target --> Connection Operation cancelled.");
                    origin_waypoint = -1;
                    src_wp = null_mut();
                } else {
                    info!("Target-waypoint {} selected. Connection established!", i);
                    (*src_wp).connections[usize::try_from((*src_wp).num_connections).unwrap()] =
                        i.try_into().unwrap();
                    (*src_wp).num_connections += 1;
                    origin_waypoint = -1;
                    src_wp = null_mut();
                }
            }

            let map_tile = &mut *((*CUR_LEVEL).map[usize::try_from(block_y).unwrap()]
                .add(block_x.try_into().unwrap()));

            // If the person using the level editor pressed some editing keys, insert the
            // corresponding map tile.  This is done here:
            if self.key_is_pressed_r(b'f'.into()) {
                *map_tile = MapTile::FineGrid as i8;
            }
            if self.key_is_pressed_r(b'1'.into()) {
                *map_tile = MapTile::Block1 as i8;
            }
            if self.key_is_pressed_r(b'2'.into()) {
                *map_tile = MapTile::Block2 as i8;
            }
            if self.key_is_pressed_r(b'3'.into()) {
                *map_tile = MapTile::Block3 as i8;
            }
            if self.key_is_pressed_r(b'4'.into()) {
                *map_tile = MapTile::Block4 as i8;
            }
            if self.key_is_pressed_r(b'5'.into()) {
                *map_tile = MapTile::Block5 as i8;
            }
            if self.key_is_pressed_r(b'l'.into()) {
                *map_tile = MapTile::Lift as i8;
            }
            if self.key_is_pressed_r(SDLK_KP_PLUS as c_int) {
                *map_tile = MapTile::VWall as i8;
            }
            if self.key_is_pressed_r(SDLK_KP0 as c_int) {
                *map_tile = MapTile::HWall as i8;
            }
            if self.key_is_pressed_r(SDLK_KP1 as c_int) {
                *map_tile = MapTile::EckLu as i8;
            }
            if self.key_is_pressed_r(SDLK_KP2 as c_int) {
                if !self.shift_pressed() {
                    *map_tile = MapTile::Tu as i8;
                } else {
                    *map_tile = MapTile::KonsoleU as i8;
                }
            }
            if self.key_is_pressed_r(SDLK_KP3 as c_int) {
                *map_tile = MapTile::EckRu as i8;
            }
            if self.key_is_pressed_r(SDLK_KP4 as c_int) {
                if !self.shift_pressed() {
                    *map_tile = MapTile::Tl as i8;
                } else {
                    *map_tile = MapTile::KonsoleL as i8;
                }
            }
            if self.key_is_pressed_r(SDLK_KP5 as c_int) {
                if !self.shift_pressed() {
                    *map_tile = MapTile::Kreuz as i8;
                } else {
                    *map_tile = MapTile::Void as i8;
                }
            }
            if self.key_is_pressed_r(SDLK_KP6 as c_int) {
                if !self.shift_pressed() {
                    *map_tile = MapTile::Tr as i8;
                } else {
                    *map_tile = MapTile::KonsoleR as i8;
                }
            }
            if self.key_is_pressed_r(SDLK_KP7 as c_int) {
                *map_tile = MapTile::EckLo as i8;
            }
            if self.key_is_pressed_r(SDLK_KP8 as c_int) {
                if !self.shift_pressed() {
                    *map_tile = MapTile::To as i8;
                } else {
                    *map_tile = MapTile::KonsoleO as i8;
                }
            }
            if self.key_is_pressed_r(SDLK_KP9 as c_int) {
                *map_tile = MapTile::EckRo as i8;
            }
            if self.key_is_pressed_r(b'm'.into()) {
                *map_tile = MapTile::AlertGreen as i8;
            }
            if self.key_is_pressed_r(b'r'.into()) {
                *map_tile = MapTile::Refresh1 as i8;
            }
            if self.key_is_pressed_r(b't'.into()) {
                if self.shift_pressed() {
                    *map_tile = MapTile::VZutuere as i8;
                } else {
                    *map_tile = MapTile::HZutuere as i8;
                }
            }
            if self.space_pressed() || self.mouse_left_pressed() {
                *map_tile = MapTile::Floor as i8;
            }
        }

        shuffle_enemys(); // now make sure droids get redestributed correctly!

        self.vars.user_rect = rect;

        clear_graph_mem();
    }
}

/// create a new empty waypoint on position x/y
fn create_waypoint(level: &mut Level, block_x: c_int, block_y: c_int) {
    if level.num_waypoints == c_int::try_from(MAXWAYPOINTS).unwrap() {
        warn!(
            "Maximal number of waypoints ({}) reached on this level. Cannot insert any more.",
            MAXWAYPOINTS,
        );
        return;
    }

    let num = usize::try_from(level.num_waypoints).unwrap();
    level.num_waypoints += 1;

    level.all_waypoints[num].x = block_x.try_into().unwrap();
    level.all_waypoints[num].y = block_y.try_into().unwrap();
    level.all_waypoints[num].num_connections = 0;
}

/// delete given waypoint num (and all its connections) on level Lev
fn delete_waypoint(level: &mut Level, num: c_int) {
    let wp_list = &mut level.all_waypoints;
    let wpmax = level.num_waypoints - 1;

    // is this the last one? then just delete
    if num == wpmax {
        wp_list[usize::try_from(num).unwrap()].num_connections = 0;
    } else {
        // otherwise shift down all higher waypoints
        let num: usize = num.try_into().unwrap();
        wp_list.copy_within((num + 1)..=usize::try_from(wpmax).unwrap(), num);
    }

    // now there's one less:
    level.num_waypoints -= 1;

    // now adjust the remaining wp-list to the changes:
    for waypoint in &mut wp_list[..usize::try_from(level.num_waypoints).unwrap()] {
        let Waypoint {
            connections,
            num_connections,
            ..
        } = waypoint;

        let mut connection_index = 0;
        while connection_index < usize::try_from(*num_connections).unwrap() {
            let connection = &mut connections[connection_index];
            // eliminate all references to this waypoint
            match (*connection).cmp(&num) {
                Ordering::Equal => {
                    // move all connections after this one down
                    connections.copy_within(
                        (connection_index + 1)..usize::try_from(*num_connections).unwrap(),
                        connection_index,
                    );
                    *num_connections -= 1;
                }
                Ordering::Greater => {
                    // adjust all connections to the shifted waypoint-numbers
                    *connection -= 1;
                    connection_index += 1;
                }
                Ordering::Less => connection_index += 1,
            }
        }
    }
}

impl Data {
    /// This function is used by the Level Editor integrated into
    /// freedroid.  It marks all waypoints with a cross.
    unsafe fn show_waypoints(&self) {
        let block_x = ME.pos.x.round();
        let block_y = ME.pos.y.round();

        SDL_LockSurface(NE_SCREEN);

        for wp in 0..usize::try_from((*CUR_LEVEL).num_waypoints).unwrap() {
            let this_wp = &mut (*CUR_LEVEL).all_waypoints[wp];
            // Draw the cross in the middle of the middle of the tile
            for i in i32::try_from(self.vars.block_rect.w / 4).unwrap()
                ..i32::try_from(3 * self.vars.block_rect.w / 4).unwrap()
            {
                // This draws a (double) line at the upper border of the current block
                let mut x =
                    i + i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w / 2)
                        - ((ME.pos.x - f32::from(this_wp.x) + 0.5)
                            * f32::from(self.vars.block_rect.w)) as i32;
                let user_center = self.get_user_center();
                let mut y = i + i32::from(user_center.y)
                    - ((ME.pos.y - f32::from(this_wp.y) + 0.5) * f32::from(self.vars.block_rect.h))
                        as i32;
                if x < i32::from(self.vars.user_rect.x)
                    || x > i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w)
                    || y < i32::from(self.vars.user_rect.y)
                    || y > i32::from(self.vars.user_rect.y) + i32::from(self.vars.user_rect.h)
                {
                    continue;
                }
                putpixel(NE_SCREEN, x, y, HIGHLIGHTCOLOR);

                x = i + i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w / 2)
                    - ((ME.pos.x - f32::from(this_wp.x) + 0.5) * f32::from(self.vars.block_rect.w))
                        as i32;
                y = i + i32::from(user_center.y)
                    - ((ME.pos.y - f32::from(this_wp.y) + 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    + 1;
                if x < i32::from(self.vars.user_rect.x)
                    || x > i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w)
                    || y < i32::from(self.vars.user_rect.y)
                    || y > i32::from(self.vars.user_rect.y) + i32::from(self.vars.user_rect.h)
                {
                    continue;
                }
                putpixel(NE_SCREEN, x, y, HIGHLIGHTCOLOR);

                // This draws a line at the lower border of the current block
                x = i + i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w / 2)
                    - ((ME.pos.x - f32::from(this_wp.x) + 0.5) * f32::from(self.vars.block_rect.w))
                        as i32;
                y = -i + i32::from(user_center.y)
                    - ((ME.pos.y - f32::from(this_wp.y) - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    - 1;
                if x < i32::from(self.vars.user_rect.x)
                    || x > i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w)
                    || y < i32::from(self.vars.user_rect.y)
                    || y > i32::from(self.vars.user_rect.y) + i32::from(self.vars.user_rect.h)
                {
                    continue;
                }
                putpixel(NE_SCREEN, x, y, HIGHLIGHTCOLOR);

                x = i + i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w / 2)
                    - ((ME.pos.x - f32::from(this_wp.x) + 0.5) * f32::from(self.vars.block_rect.w))
                        as i32;
                y = -i + i32::from(user_center.y)
                    - ((ME.pos.y - f32::from(this_wp.y) - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    - 2;
                if x < i32::from(self.vars.user_rect.x)
                    || x > i32::from(self.vars.user_rect.x) + i32::from(self.vars.user_rect.w)
                    || y < i32::from(self.vars.user_rect.y)
                    || y > i32::from(self.vars.user_rect.y) + i32::from(self.vars.user_rect.h)
                {
                    continue;
                }
                putpixel(NE_SCREEN, x, y, HIGHLIGHTCOLOR);
            }

            // Draw the connections to other waypoints, BUT ONLY FOR THE WAYPOINT CURRENTLY TARGETED
            if (block_x - f32::from(this_wp.x)).abs() <= f32::EPSILON
                && (block_y - f32::from(this_wp.y)).abs() <= f32::EPSILON
            {
                for &connection in
                    &this_wp.connections[0..usize::try_from(this_wp.num_connections).unwrap()]
                {
                    let connection = usize::try_from(connection).unwrap();
                    self.draw_line_between_tiles(
                        this_wp.x.into(),
                        this_wp.y.into(),
                        (*CUR_LEVEL).all_waypoints[connection].x.into(),
                        (*CUR_LEVEL).all_waypoints[connection].y.into(),
                        HIGHLIGHTCOLOR as i32,
                    );
                }
            }
        }

        SDL_UnlockSurface(NE_SCREEN);
    }

    /// This function is used by the Level Editor integrated into
    /// freedroid.  It highlights the map position that is currently
    /// edited or would be edited, if the user pressed something.  I.e.
    /// it provides a "cursor" for the Level Editor.
    unsafe fn highlight_current_block(&self) {
        SDL_LockSurface(NE_SCREEN);

        let user_center = self.get_user_center();
        for i in 0..i32::from(self.vars.block_rect.w) {
            // This draws a (double) line at the upper border of the current block
            putpixel(
                NE_SCREEN,
                i + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x - 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32,
                HIGHLIGHTCOLOR,
            );
            putpixel(
                NE_SCREEN,
                i + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x - 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    + 1,
                HIGHLIGHTCOLOR,
            );

            // This draws a line at the lower border of the current block
            putpixel(
                NE_SCREEN,
                i + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x - 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y + 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    - 1,
                HIGHLIGHTCOLOR,
            );
            putpixel(
                NE_SCREEN,
                i + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x - 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y + 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    - 2,
                HIGHLIGHTCOLOR,
            );

            // This draws a line at the left border of the current block
            putpixel(
                NE_SCREEN,
                i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x - 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    + i,
                HIGHLIGHTCOLOR,
            );
            putpixel(
                NE_SCREEN,
                1 + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x - 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    + i,
                HIGHLIGHTCOLOR,
            );

            // This draws a line at the right border of the current block
            putpixel(
                NE_SCREEN,
                -1 + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x + 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    + i,
                HIGHLIGHTCOLOR,
            );
            putpixel(
                NE_SCREEN,
                -2 + i32::from(self.vars.user_rect.x)
                    + i32::from(self.vars.user_rect.w / 2)
                    + (((ME.pos.x).round() - ME.pos.x + 0.5) * f32::from(self.vars.block_rect.w))
                        as i32,
                i32::from(user_center.y)
                    + (((ME.pos.y).round() - ME.pos.y - 0.5) * f32::from(self.vars.block_rect.h))
                        as i32
                    + i,
                HIGHLIGHTCOLOR,
            );
        }

        SDL_UnlockSurface(NE_SCREEN);
    }
}
