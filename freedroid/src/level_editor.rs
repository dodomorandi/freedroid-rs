use crate::{
    b_font::{font_height, print_string_font},
    cur_level,
    defs::{
        AssembleCombatWindowFlags, Cmds, MapTile, MAXWAYPOINTS, MAX_WP_CONNECTIONS, NUM_MAP_BLOCKS,
    },
    structs::{Level, Waypoint},
    view::BLACK,
};

use log::{info, warn};
use nom::Finish;
use sdl::{convert::u32_to_u16, Pixel};
use sdl_sys::{
    SDLKey_SDLK_F1, SDLKey_SDLK_KP0, SDLKey_SDLK_KP1, SDLKey_SDLK_KP2, SDLKey_SDLK_KP3,
    SDLKey_SDLK_KP4, SDLKey_SDLK_KP5, SDLKey_SDLK_KP6, SDLKey_SDLK_KP7, SDLKey_SDLK_KP8,
    SDLKey_SDLK_KP9, SDLKey_SDLK_KP_PLUS,
};
use std::{cmp::Ordering, ops::Not};

const HIGHLIGHTCOLOR: Pixel = Pixel::from_u8(255);
const HIGHLIGHTCOLOR2: Pixel = Pixel::from_u8(100);

/// create a new empty waypoint on position x/y
fn create_waypoint(level: &mut Level, block_x: i32, block_y: i32) {
    if level.num_waypoints == i32::try_from(MAXWAYPOINTS).unwrap() {
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
fn delete_waypoint(level: &mut Level, num: i32) {
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

impl crate::Data<'_> {
    /// This function is provides the Level Editor integrated into
    /// freedroid.  Actually this function is a submenu of the big
    /// Escape Menu.  In here you can edit the level and upon pressing
    /// escape enter a further submenu where you can save the level,
    /// change level name and quit from level editing.
    pub fn level_editor(&mut self) {
        let mut done = false;
        let mut origin_waypoint: i32 = -1;

        let rect = self.vars.user_rect;
        self.vars.user_rect = self.vars.screen_rect; // level editor can use the full screen!
        let mut src_wp_index = None;

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

            #[allow(clippy::cast_possible_truncation)]
            let [block_x, block_y] = [
                (self.vars.me.pos.x).round() as i32,
                (self.vars.me.pos.y).round() as i32,
            ];

            self.fill_rect(self.vars.user_rect, BLACK);
            self.assemble_combat_picture(AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits().into());
            self.highlight_current_block();
            self.show_waypoints();

            // show line between a selected connection-origin and the current block
            if origin_waypoint != -1 {
                #[allow(clippy::cast_precision_loss)]
                self.draw_line_between_tiles(
                    block_x as f32,
                    block_y as f32,
                    self.main.cur_level().all_waypoints[usize::try_from(origin_waypoint).unwrap()]
                        .x
                        .into(),
                    self.main.cur_level().all_waypoints[usize::try_from(origin_waypoint).unwrap()]
                        .y
                        .into(),
                    HIGHLIGHTCOLOR2,
                );
            }

            let font0 = self
                .global
                .font0_b_font
                .as_ref()
                .unwrap()
                .rw(&mut self.font_owner);

            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0,
                i32::from(self.vars.full_user_rect.x())
                    + i32::from(self.vars.full_user_rect.width()) / 3,
                i32::from(self.vars.full_user_rect.y())
                    + i32::from(self.vars.full_user_rect.height())
                    - font_height(&*font0),
                format_args!("Press F1 for keymap"),
            );

            assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

            self.handle_level_editor_arrow_keys();

            if self.key_is_pressed_r(SDLKey_SDLK_F1.try_into().unwrap()) {
                self.handle_level_editor_help();
            }

            //--------------------
            // Since the level editor will not always be able to
            // immediately feature all the the map tiles that might
            // have been added recently, we should offer a feature, so that you can
            // specify the value of a map piece just numerically.  This will be
            // done upon pressing the 'e' key.
            //
            if self.key_is_pressed_r(b'e'.into()) {
                self.handle_level_editor_tile_by_number(block_x, block_y);
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
                self.handle_level_editor_toggle_waypoint(block_x, block_y);
            }

            // create a connection between waypoints.  If this is the first selected waypoint, its
            // an origin and the second "C"-pressed waypoint will be used a target.
            // If origin and destination are the same, the operation is cancelled.
            if self.key_is_pressed_r(b'c'.into()) {
                self.handle_level_editor_waypoint_connection(
                    block_x,
                    block_y,
                    &mut origin_waypoint,
                    &mut src_wp_index,
                );
            }

            // If the person using the level editor pressed some editing keys, insert the
            // corresponding map tile.  This is done here:
            let map_tile = self.handle_level_editor_key_pressed();

            if let Some(map_tile) = map_tile {
                self.main.cur_level_mut().map[usize::try_from(block_y).unwrap()]
                    [usize::try_from(block_x).unwrap()] = map_tile;
            }
        }

        self.shuffle_enemys(); // now make sure droids get redestributed correctly!

        self.vars.user_rect = rect;

        self.clear_graph_mem();
    }

    fn handle_level_editor_arrow_keys(&mut self) {
        if self.left_pressed_r() && self.vars.me.pos.x.round() > 0. {
            self.vars.me.pos.x -= 1.;
        }

        #[allow(clippy::cast_possible_truncation)]
        if self.right_pressed_r()
            && (self.vars.me.pos.x.round() as i32) < self.main.cur_level().xlen - 1
        {
            self.vars.me.pos.x += 1.;
        }

        if self.up_pressed_r() && self.vars.me.pos.y.round() > 0. {
            self.vars.me.pos.y -= 1.;
        }

        #[allow(clippy::cast_possible_truncation)]
        if self.down_pressed_r()
            && (self.vars.me.pos.y.round() as i32) < self.main.cur_level().ylen - 1
        {
            self.vars.me.pos.y += 1.;
        }
    }

    fn handle_level_editor_help(&mut self) {
        const KEYMAP_OFFSET: i32 = 15;

        let mut k = 3;
        self.make_grid_on_screen(None);
        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        let menu_b_font = self
            .global
            .menu_b_font
            .as_ref()
            .unwrap()
            .ro(&self.font_owner);
        let font_height = font_height(menu_b_font);

        self.centered_put_string(&mut ne_screen, k * font_height, b"Level Editor Keymap");
        macro_rules! put_string {
            ($s:expr) => {
                self.put_string(&mut ne_screen, KEYMAP_OFFSET, k * font_height, $s);
            };
        }

        k += 2;
        put_string!(b"Use cursor keys to move around.");
        k += 1;
        put_string!(b"Use number pad to plant walls.");
        k += 1;
        put_string!(b"Use shift and number pad to plant extras.");
        k += 1;
        put_string!(b"R...Refresh, 1-5...Blocktype 1-5, L...Lift");
        k += 1;
        put_string!(b"F...Fine grid, T/SHIFT + T...Doors");
        k += 1;
        put_string!(b"M...Alert, E...Enter tile by number");
        k += 1;
        put_string!(b"Space/Enter...Floor");
        k += 2;

        put_string!(b"I/O...zoom INTO/OUT OF the map");
        k += 2;
        put_string!(b"P...toggle wayPOINT on/off");
        k += 1;
        put_string!(b"C...start/end waypoint CONNECTION");

        assert!(ne_screen.flip());
        self.graphics.ne_screen = Some(ne_screen);
        while !self.fire_pressed_r() && !self.escape_pressed_r() && !self.return_pressed_r() {
            self.sdl.delay_ms(1);
        }
    }

    fn handle_level_editor_tile_by_number(&mut self, block_x: i32, block_y: i32) {
        use nom::{
            character::complete::{i32, space0},
            sequence::preceded,
        };

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        let menu_b_font = self
            .global
            .menu_b_font
            .as_ref()
            .unwrap()
            .ro(&self.font_owner);
        let font_height = font_height(menu_b_font);

        Self::centered_put_string_static(
            &self.b_font,
            &mut self.font_owner,
            &mut ne_screen,
            6 * font_height,
            b"Please enter new value: ",
        );
        assert!(ne_screen.flip());
        self.graphics.ne_screen = Some(ne_screen);
        let numeric_input_string = self.get_string(10, 2).unwrap();

        let mut special_map_value = preceded(space0::<_, ()>, i32)(numeric_input_string.as_bytes())
            .finish()
            .unwrap()
            .1;
        if special_map_value >= NUM_MAP_BLOCKS.try_into().unwrap() {
            special_map_value = 0;
        }
        self.main.cur_level_mut().map[usize::try_from(block_y).unwrap()]
            [usize::try_from(block_x).unwrap()] = special_map_value.try_into().unwrap();
    }

    fn handle_level_editor_toggle_waypoint(&mut self, block_x: i32, block_y: i32) {
        // find out if there is a waypoint on the current square
        let mut i = 0;
        while i < usize::try_from(self.main.cur_level().num_waypoints).unwrap() {
            if i32::from(self.main.cur_level().all_waypoints[i].x) == block_x
                && i32::from(self.main.cur_level().all_waypoints[i].y) == block_y
            {
                break;
            }

            i += 1;
        }

        // if its waypoint already, this waypoint must be deleted.
        if i < usize::try_from(self.main.cur_level().num_waypoints).unwrap() {
            delete_waypoint(self.main.cur_level_mut(), i.try_into().unwrap());
        } else {
            // if its not a waypoint already, it must be made into one
            create_waypoint(self.main.cur_level_mut(), block_x, block_y);
        }
    }

    fn handle_level_editor_waypoint_connection(
        &mut self,
        block_x: i32,
        block_y: i32,
        origin_waypoint: &mut i32,
        src_wp_index: &mut Option<usize>,
    ) {
        // Determine which waypoint is currently targeted
        let mut i = 0;
        while i < usize::try_from(self.main.cur_level().num_waypoints).unwrap() {
            if i32::from(self.main.cur_level().all_waypoints[i].x) == block_x
                && i32::from(self.main.cur_level().all_waypoints[i].y) == block_y
            {
                break;
            }

            i += 1;
        }

        if i == usize::try_from(self.main.cur_level().num_waypoints).unwrap() {
            warn!("Sorry, no waypoint here to connect.");
        } else if *origin_waypoint == -1 {
            *origin_waypoint = i.try_into().unwrap();
            let waypoint = &mut cur_level!(mut self.main).all_waypoints[i];
            if waypoint.num_connections < i32::try_from(MAX_WP_CONNECTIONS).unwrap() {
                info!("Waypoint nr. {}. selected as origin", i);
                *src_wp_index = Some(i);
            } else {
                warn!(
                    "Sorry, maximal number of waypoint-connections ({}) reached! Operation \
                             not possible.",
                    MAX_WP_CONNECTIONS,
                );
                *origin_waypoint = -1;
                *src_wp_index = None;
            }
        } else if *origin_waypoint == i32::try_from(i).unwrap() {
            info!("Origin==Target --> Connection Operation cancelled.");
            *origin_waypoint = -1;
            *src_wp_index = None;
        } else {
            info!("Target-waypoint {} selected. Connection established!", i);
            let waypoint_index = src_wp_index.take().unwrap();
            let waypoint = &mut cur_level!(mut self.main).all_waypoints[waypoint_index];
            waypoint.connections[usize::try_from(waypoint.num_connections).unwrap()] =
                i.try_into().unwrap();
            waypoint.num_connections += 1;
            *origin_waypoint = -1;
        }
    }

    fn handle_level_editor_key_pressed(&mut self) -> Option<MapTile> {
        let mut map_tile = None;
        if self.key_is_pressed_r(b'f'.into()) {
            map_tile = Some(MapTile::FineGrid);
        }
        if self.key_is_pressed_r(b'1'.into()) {
            map_tile = Some(MapTile::Block1);
        }
        if self.key_is_pressed_r(b'2'.into()) {
            map_tile = Some(MapTile::Block2);
        }
        if self.key_is_pressed_r(b'3'.into()) {
            map_tile = Some(MapTile::Block3);
        }
        if self.key_is_pressed_r(b'4'.into()) {
            map_tile = Some(MapTile::Block4);
        }
        if self.key_is_pressed_r(b'5'.into()) {
            map_tile = Some(MapTile::Block5);
        }
        if self.key_is_pressed_r(b'l'.into()) {
            map_tile = Some(MapTile::Lift);
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP_PLUS)) {
            map_tile = Some(MapTile::VWall);
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP0)) {
            map_tile = Some(MapTile::HWall);
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP1)) {
            map_tile = Some(MapTile::EckLu);
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP2)) {
            if self.shift_pressed() {
                map_tile = Some(MapTile::KonsoleU);
            } else {
                map_tile = Some(MapTile::Tu);
            }
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP3)) {
            map_tile = Some(MapTile::EckRu);
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP4)) {
            if self.shift_pressed() {
                map_tile = Some(MapTile::KonsoleL);
            } else {
                map_tile = Some(MapTile::Tl);
            }
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP5)) {
            if self.shift_pressed() {
                map_tile = Some(MapTile::Void);
            } else {
                map_tile = Some(MapTile::Kreuz);
            }
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP6)) {
            if self.shift_pressed() {
                map_tile = Some(MapTile::KonsoleR);
            } else {
                map_tile = Some(MapTile::Tr);
            }
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP7)) {
            map_tile = Some(MapTile::EckLo);
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP8)) {
            if self.shift_pressed() {
                map_tile = Some(MapTile::KonsoleO);
            } else {
                map_tile = Some(MapTile::To);
            }
        }
        if self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_KP9)) {
            map_tile = Some(MapTile::EckRo);
        }
        if self.key_is_pressed_r(b'm'.into()) {
            map_tile = Some(MapTile::AlertGreen);
        }
        if self.key_is_pressed_r(b'r'.into()) {
            map_tile = Some(MapTile::Refresh1);
        }
        if self.key_is_pressed_r(b't'.into()) {
            if self.shift_pressed() {
                map_tile = Some(MapTile::VZutuere);
            } else {
                map_tile = Some(MapTile::HZutuere);
            }
        }
        if self.space_pressed() || self.mouse_left_pressed() {
            map_tile = Some(MapTile::Floor);
        }

        map_tile
    }

    /// This function is used by the Level Editor integrated into
    /// freedroid.  It marks all waypoints with a cross.
    fn show_waypoints(&mut self) {
        let block_x = self.vars.me.pos.x.round();
        let block_y = self.vars.me.pos.y.round();

        let user_rect_x = i32::from(self.vars.user_rect.x());
        let user_rect_y = i32::from(self.vars.user_rect.y());
        let user_rect_width = i32::from(self.vars.user_rect.width());
        let user_rect_height = i32::from(self.vars.user_rect.height());
        let block_rect_width_f = f32::from(self.vars.block_rect.width());
        let block_rect_height_f = f32::from(self.vars.block_rect.height());

        for wp in 0..usize::try_from(self.main.cur_level().num_waypoints).unwrap() {
            let this_wp = &cur_level!(self.main).all_waypoints[wp];
            let wp_x = f32::from(this_wp.x);
            let wp_y = f32::from(this_wp.y);

            // Draw the cross in the middle of the middle of the tile
            for i in i32::from(self.vars.block_rect.width() / 4)
                ..i32::from(3 * self.vars.block_rect.width() / 4)
            {
                // This draws a (double) line at the upper border of the current block
                #[allow(clippy::cast_possible_truncation)]
                let mut x = i + user_rect_x + user_rect_width / 2
                    - ((self.vars.me.pos.x - wp_x + 0.5) * block_rect_width_f) as i32;
                let user_center = self.vars.get_user_center();
                #[allow(clippy::cast_possible_truncation)]
                let mut y = i + i32::from(user_center.y())
                    - ((self.vars.me.pos.y - wp_y + 0.5) * block_rect_height_f) as i32;

                macro_rules! check_valid_x_y {
                    () => {
                        if x < user_rect_x
                            || x >= user_rect_x + user_rect_width
                            || y < user_rect_y
                            || y >= user_rect_y + user_rect_height
                        {
                            continue;
                        }
                    };
                }
                check_valid_x_y!();

                // FIXME: avoid this inside the loop
                let mut ne_screen = self.graphics.ne_screen.as_mut().unwrap().lock().unwrap();
                ne_screen
                    .pixels()
                    .set(x.try_into().unwrap(), y.try_into().unwrap(), HIGHLIGHTCOLOR)
                    .unwrap();

                #[allow(clippy::cast_possible_truncation)]
                {
                    x = i + user_rect_x + user_rect_width / 2
                        - ((self.vars.me.pos.x - wp_x + 0.5) * block_rect_width_f) as i32;
                    y = i + i32::from(user_center.y())
                        - ((self.vars.me.pos.y - wp_y + 0.5) * block_rect_height_f) as i32
                        + 1;
                }
                check_valid_x_y!();

                ne_screen
                    .pixels()
                    .set(x.try_into().unwrap(), y.try_into().unwrap(), HIGHLIGHTCOLOR)
                    .unwrap();

                // This draws a line at the lower border of the current block
                #[allow(clippy::cast_possible_truncation)]
                {
                    x = i + user_rect_x + user_rect_width / 2
                        - ((self.vars.me.pos.x - wp_x + 0.5) * block_rect_width_f) as i32;
                    y = -i + i32::from(user_center.y())
                        - ((self.vars.me.pos.y - wp_y - 0.5) * block_rect_height_f) as i32
                        - 1;
                }
                check_valid_x_y!();

                ne_screen
                    .pixels()
                    .set(x.try_into().unwrap(), y.try_into().unwrap(), HIGHLIGHTCOLOR)
                    .unwrap();

                #[allow(clippy::cast_possible_truncation)]
                {
                    x = i + user_rect_x + user_rect_width / 2
                        - ((self.vars.me.pos.x - wp_x + 0.5) * block_rect_width_f) as i32;
                    y = -i + i32::from(user_center.y())
                        - ((self.vars.me.pos.y - wp_y - 0.5) * block_rect_height_f) as i32
                        - 2;
                }
                check_valid_x_y!();

                ne_screen
                    .pixels()
                    .set(x.try_into().unwrap(), y.try_into().unwrap(), HIGHLIGHTCOLOR)
                    .unwrap();
            }

            // Draw the connections to other waypoints, BUT ONLY FOR THE WAYPOINT CURRENTLY TARGETED
            if (block_x - wp_x).abs() <= f32::EPSILON && (block_y - wp_y).abs() <= f32::EPSILON {
                for &connection in
                    &this_wp.connections[0..usize::try_from(this_wp.num_connections).unwrap()]
                {
                    let connection = usize::try_from(connection).unwrap();
                    let waypoint = &cur_level!(self.main).all_waypoints[connection];
                    Self::draw_line_between_tiles_static(
                        &self.vars,
                        &mut self.graphics,
                        this_wp.x.into(),
                        this_wp.y.into(),
                        waypoint.x.into(),
                        waypoint.y.into(),
                        HIGHLIGHTCOLOR,
                    );
                }
            }
        }
    }

    /// This function is used by the Level Editor integrated into
    /// freedroid.  It highlights the map position that is currently
    /// edited or would be edited, if the user pressed something.  I.e.
    /// it provides a "cursor" for the Level Editor.
    #[allow(clippy::similar_names)]
    fn highlight_current_block(&mut self) {
        let mut ne_screen = self.graphics.ne_screen.as_mut().unwrap().lock().unwrap();
        let mut pixels = ne_screen.pixels();

        macro_rules! set_pixels {
            ($x:expr, $y:expr $(,)?) => {
                #[allow(clippy::cast_possible_truncation)]
                pixels
                    .set(
                        u16::try_from($x).unwrap(),
                        u16::try_from($y).unwrap(),
                        HIGHLIGHTCOLOR,
                    )
                    .unwrap();
            };
        }

        let user_rect_x = i32::from(self.vars.user_rect.x());
        let user_rect_width = i32::from(self.vars.user_rect.width());
        let block_rect_width = f32::from(self.vars.block_rect.width());
        let block_rect_height = f32::from(self.vars.block_rect.height());
        let pos_x_r = (self.vars.me.pos.x).round();
        let pos_y_r = (self.vars.me.pos.y).round();

        let user_center = self.vars.get_user_center();
        let user_center_y = i32::from(user_center.y());
        for i in 0..i32::from(self.vars.block_rect.width()) {
            // This draws a (double) line at the upper border of the current block
            set_pixels!(
                i + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x - 0.5) * block_rect_width) as i32,
                user_center_y + ((pos_y_r - self.vars.me.pos.y - 0.5) * block_rect_height) as i32,
            );
            set_pixels!(
                i + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x - 0.5) * block_rect_width) as i32,
                user_center_y
                    + ((pos_y_r - self.vars.me.pos.y - 0.5) * block_rect_height) as i32
                    + 1,
            );

            // This draws a line at the lower border of the current block
            set_pixels!(
                i + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x - 0.5) * block_rect_width) as i32,
                user_center_y + ((pos_y_r - self.vars.me.pos.y + 0.5) * block_rect_height) as i32
                    - 1,
            );
            set_pixels!(
                i + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x - 0.5) * block_rect_width) as i32,
                user_center_y + ((pos_y_r - self.vars.me.pos.y + 0.5) * block_rect_height) as i32
                    - 2,
            );

            // This draws a line at the left border of the current block
            set_pixels!(
                user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x - 0.5) * block_rect_width) as i32,
                user_center_y
                    + ((pos_y_r - self.vars.me.pos.y - 0.5) * block_rect_height) as i32
                    + i,
            );
            set_pixels!(
                1 + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x - 0.5) * block_rect_width) as i32,
                user_center_y
                    + ((pos_y_r - self.vars.me.pos.y - 0.5) * block_rect_height) as i32
                    + i,
            );

            // This draws a line at the right border of the current block
            set_pixels!(
                -1 + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x + 0.5) * block_rect_width) as i32,
                user_center_y
                    + ((pos_y_r - self.vars.me.pos.y - 0.5) * block_rect_height) as i32
                    + i,
            );
            set_pixels!(
                -2 + user_rect_x
                    + user_rect_width / 2
                    + ((pos_x_r - self.vars.me.pos.x + 0.5) * block_rect_width) as i32,
                user_center_y
                    + ((pos_y_r - self.vars.me.pos.y - 0.5) * block_rect_height) as i32
                    + i,
            );
        }
    }
}
