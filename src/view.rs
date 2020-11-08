use crate::{
    b_font::{FontHeight, PrintStringFont, PutStringFont},
    defs::{self, get_user_center, AssembleCombatWindowFlags, Status, MAXBLASTS, MAXBULLETS},
    global::{
        ne_screen, show_all_droids, AllBlasts, AllBullets, AllEnemys, Black, Block_Rect,
        BuildBlock, CurLevel, DeathCount, Decal_pics, Druidmap, EnemyDigitSurfacePointer,
        EnemySurfacePointer, FirstDigit_Rect, Font0_BFont, Full_User_Rect, GameConfig,
        MapBlockSurfacePointer, Me, Number_Of_Droid_Types, SecondDigit_Rect, ThirdDigit_Rect,
        User_Rect,
    },
    map::{GetMapBrick, IsVisible},
    misc::{Frame_Time, Terminate},
    structs::{Enemy, Finepoint, GrobPoint},
};

use cstr::cstr;
use log::{error, info, trace};
use sdl::{
    sdl::Rect,
    video::ll::{
        SDL_Color, SDL_FillRect, SDL_MapRGB, SDL_SetClipRect, SDL_UpdateRect, SDL_UpperBlit,
    },
};
use std::{
    cell::Cell,
    convert::{TryFrom, TryInto},
    os::raw::c_int,
    ptr::null_mut,
};

extern "C" {
    #[no_mangle]
    pub fn PutInfluence(x: c_int, y: c_int);

    #[no_mangle]
    pub fn PutBullet(bullets_number: c_int);

    #[no_mangle]
    pub fn PutBlast(blasts_number: c_int);
}

#[no_mangle]
pub unsafe extern "C" fn Fill_Rect(mut rect: Rect, color: SDL_Color) {
    let pixcolor = SDL_MapRGB((*ne_screen).format, color.r, color.g, color.b);

    SDL_FillRect(ne_screen, &mut rect, pixcolor);
}

/// This function assembles the contents of the combat window
/// in ne_screen.
///
/// Several FLAGS can be used to control its behaviour:
///
/// (*) ONLY_SHOW_MAP = 0x01:  This flag indicates not do draw any
///     game elements but the map blocks
///
/// (*) DO_SCREEgN_UPDATE = 0x02: This flag indicates for the function
///     to also cause an SDL_Update of the portion of the screen
///     that has been modified
///
/// (*) SHOW_FULL_MAP = 0x04: show complete map, disregard visibility
#[no_mangle]
pub unsafe extern "C" fn Assemble_Combat_Picture(mask: c_int) {
    thread_local! {
        static TIME_SINCE_LAST_FPS_UPDATE: Cell<f32> = Cell::new(10.);
        static FPS_DISPLAYED: Cell<i32>=Cell::new(1);
    }

    const UPDATE_FPS_HOW_OFTEN: f32 = 0.75;

    info!("\nvoid Assemble_Combat_Picture(...): Real function call confirmed.");

    SDL_SetClipRect(ne_screen, &User_Rect);

    if GameConfig.AllMapVisible == 0 {
        Fill_Rect(User_Rect, Black);
    }

    let (upleft, downright) =
        if (mask & AssembleCombatWindowFlags::SHOW_FULL_MAP.bits() as i32) != 0 {
            let upleft = GrobPoint { x: -5, y: -5 };
            let downright = GrobPoint {
                x: (*CurLevel).xlen as i8 + 5,
                y: (*CurLevel).ylen as i8 + 5,
            };
            (upleft, downright)
        } else {
            let upleft = GrobPoint {
                x: Me.pos.x as i8 - 6,
                y: Me.pos.y as i8 - 5,
            };
            let downright = GrobPoint {
                x: Me.pos.x as i8 + 7,
                y: Me.pos.y as i8 + 5,
            };
            (upleft, downright)
        };

    let mut pos = Finepoint::default();
    let mut vect = Finepoint::default();
    let mut len = -1f32;
    let mut map_brick = 0;
    let mut target_rectangle = Rect::new(0, 0, 0, 0);
    (upleft.y..downright.y)
        .flat_map(|line| (upleft.x..downright.x).map(move |col| (line, col)))
        .for_each(|(line, col)| {
            if GameConfig.AllMapVisible == 0
                && ((mask & AssembleCombatWindowFlags::SHOW_FULL_MAP.bits() as i32) == 0x0)
            {
                pos.x = col.into();
                pos.y = line.into();
                vect.x = Me.pos.x - pos.x;
                vect.y = Me.pos.y - pos.y;
                len = (vect.x * vect.x + vect.y * vect.y).sqrt() + 0.01;
                vect.x /= len;
                vect.y /= len;
                if len > 0.5 {
                    pos.x += vect.x;
                    pos.y += vect.y;
                }
                if IsVisible(&mut pos) == 0 {
                    return;
                }
            }

            map_brick = GetMapBrick(CurLevel, col.into(), line.into());
            let user_center = get_user_center();
            target_rectangle.x = user_center.x
                + ((-Me.pos.x + 1.0 * f32::from(col) - 0.5) * f32::from(Block_Rect.w)).round()
                    as i16;
            target_rectangle.y = user_center.y
                + ((-Me.pos.y + 1.0 * f32::from(line) - 0.5) * f32::from(Block_Rect.h)).round()
                    as i16;
            SDL_UpperBlit(
                MapBlockSurfacePointer[usize::try_from((*CurLevel).color).unwrap()]
                    [usize::from(map_brick)],
                null_mut(),
                ne_screen,
                &mut target_rectangle,
            );
        });

    // if we don't use Fullscreen mode, we have to clear the text-background manually
    // for the info-line text:

    let mut text_rect = Rect::new(
        Full_User_Rect.x,
        (i32::from(Full_User_Rect.y) + i32::from(Full_User_Rect.h)
            - i32::from(FontHeight(&*Font0_BFont)))
        .try_into()
        .unwrap(),
        Full_User_Rect.w,
        FontHeight(&*Font0_BFont).try_into().unwrap(),
    );
    SDL_SetClipRect(ne_screen, &text_rect);
    if GameConfig.FullUserRect == 0 {
        SDL_FillRect(ne_screen, &mut text_rect, 0);
    }

    if GameConfig.Draw_Position != 0 {
        PrintStringFont(
            ne_screen,
            Font0_BFont,
            (Full_User_Rect.x + (Full_User_Rect.w / 6) as i16).into(),
            i32::from(Full_User_Rect.y) + i32::from(Full_User_Rect.h)
                - i32::from(FontHeight(&*Font0_BFont)),
            cstr!("GPS: X=%d Y=%d Lev=%d").as_ptr() as *mut i8,
            (Me.pos.x).round() as i32,
            (Me.pos.y).round() as i32,
            (*CurLevel).levelnum,
        );
    }

    if mask & AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits() as i32 == 0 {
        if GameConfig.Draw_Framerate != 0 {
            TIME_SINCE_LAST_FPS_UPDATE.with(|time_cell| {
                let mut time = time_cell.get();
                time += Frame_Time();

                if time > UPDATE_FPS_HOW_OFTEN {
                    FPS_DISPLAYED.with(|fps_displayed| {
                        fps_displayed.set((1.0 / Frame_Time()) as i32);
                    });
                    time_cell.set(0.);
                } else {
                    time_cell.set(time);
                }
            });

            FPS_DISPLAYED.with(|fps_displayed| {
                PrintStringFont(
                    ne_screen,
                    Font0_BFont,
                    Full_User_Rect.x.into(),
                    (Full_User_Rect.y as i32 + Full_User_Rect.h as i32
                        - FontHeight(&*Font0_BFont) as i32)
                        .into(),
                    cstr!("FPS: %d ").as_ptr() as *mut i8,
                    fps_displayed.get(),
                );
            });
        }

        if GameConfig.Draw_Energy != 0 {
            PrintStringFont(
                ne_screen,
                Font0_BFont,
                i32::from(Full_User_Rect.x) + i32::from(Full_User_Rect.w) / 2,
                i32::from(Full_User_Rect.y) + i32::from(Full_User_Rect.h)
                    - i32::from(FontHeight(&*Font0_BFont)),
                cstr!("Energy: %d").as_ptr() as *mut i8,
                Me.energy as i32,
            );
        }
        if GameConfig.Draw_DeathCount != 0 {
            PrintStringFont(
                ne_screen,
                Font0_BFont,
                i32::from(Full_User_Rect.x) + 2 * i32::from(Full_User_Rect.w) / 3,
                i32::from(Full_User_Rect.y) + i32::from(Full_User_Rect.h)
                    - i32::from(FontHeight(&*Font0_BFont)),
                cstr!("Deathcount: %d").as_ptr() as *mut i8,
                DeathCount as i32,
            );
        }

        SDL_SetClipRect(ne_screen, &User_Rect);

        // make sure Ashes are displayed _before_ droids, so that they are _under_ them!
        for enemy in &mut AllEnemys {
            if (enemy.status == Status::Terminated as i32)
                && (enemy.levelnum == (*CurLevel).levelnum)
            {
                if IsVisible(&mut enemy.pos) != 0 {
                    PutAshes(enemy.pos.x, enemy.pos.y);
                }
            }
        }

        AllEnemys
            .iter()
            .enumerate()
            .filter(|(_, enemy)| {
                !((enemy.levelnum != (*CurLevel).levelnum)
                    || (enemy.status == Status::Out as i32)
                    || (enemy.status == Status::Terminated as i32))
            })
            .for_each(|(index, _)| PutEnemy(index as c_int, -1, -1));

        if Me.energy > 0. {
            PutInfluence(-1, -1);
        }

        AllBullets
            .iter()
            .take(MAXBULLETS)
            .enumerate()
            .filter(|(_, bullet)| bullet.ty != Status::Out as u8)
            .for_each(|(index, _)| PutBullet(index as i32));

        AllBlasts
            .iter()
            .take(MAXBLASTS)
            .enumerate()
            .filter(|(_, blast)| blast.ty != Status::Out as i32)
            .for_each(|(index, _)| PutBlast(index as i32));
    }

    // At this point we are done with the drawing procedure
    // and all that remains to be done is updating the screen.

    if mask & AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits() as i32 != 0 {
        SDL_UpdateRect(
            ne_screen,
            User_Rect.x.into(),
            User_Rect.y.into(),
            User_Rect.w.into(),
            User_Rect.h.into(),
        );
        SDL_UpdateRect(
            ne_screen,
            text_rect.x.into(),
            text_rect.y.into(),
            text_rect.w.into(),
            text_rect.h.into(),
        );
    }

    SDL_SetClipRect(ne_screen, null_mut());
}

/// put some ashes at (x,y)
#[no_mangle]
pub unsafe extern "C" fn PutAshes(x: f32, y: f32) {
    if GameConfig.ShowDecals == 0 {
        return;
    }

    let user_center = get_user_center();
    let mut dst = Rect::new(
        (f32::from(user_center.x) + (-Me.pos.x + x) * f32::from(Block_Rect.w)
            - f32::from(Block_Rect.w / 2)) as i16,
        (f32::from(user_center.y) + (-Me.pos.y + y) * f32::from(Block_Rect.h)
            - f32::from(Block_Rect.h / 2)) as i16,
        0,
        0,
    );
    SDL_UpperBlit(Decal_pics[0], null_mut(), ne_screen, &mut dst);
}

#[no_mangle]
pub unsafe extern "C" fn PutEnemy(enemy_index: c_int, x: c_int, y: c_int) {
    let droid: &mut Enemy = &mut AllEnemys[usize::try_from(enemy_index).unwrap()];
    let ty = droid.ty;
    let phase = droid.phase;
    let name = &mut (&mut *Druidmap.offset(ty.try_into().unwrap())).druidname;

    if (droid.status == Status::Terminated as i32)
        || (droid.status == Status::Out as i32)
        || (droid.levelnum != (*CurLevel).levelnum)
    {
        return;
    }

    // if the enemy is out of sight, we need not do anything more here
    if show_all_droids == 0 && IsVisible(&mut droid.pos) == 0 {
        trace!("ONSCREEN=FALSE --> usual end of function reached.");
        return;
    }

    // We check for incorrect droid types, which sometimes might occor, especially after
    // heavy editing of the crew initialisation functions ;)
    if droid.ty >= Number_Of_Droid_Types {
        error!("nonexistant droid-type encountered: {}", droid.ty);
        Terminate(defs::ERR.into());
    }

    //--------------------
    // First blit just the enemy hat and shoes.
    SDL_UpperBlit(
        EnemySurfacePointer[phase as usize],
        null_mut(),
        BuildBlock,
        null_mut(),
    );

    //--------------------
    // Now the numbers should be blittet.
    let mut dst = FirstDigit_Rect.clone();
    SDL_UpperBlit(
        EnemyDigitSurfacePointer[usize::try_from(name[0] - b'1' as i8 + 1).unwrap()],
        null_mut(),
        BuildBlock,
        &mut dst,
    );

    dst = SecondDigit_Rect.clone();
    SDL_UpperBlit(
        EnemyDigitSurfacePointer[usize::try_from(name[1] - b'1' as i8 + 1).unwrap()],
        null_mut(),
        BuildBlock,
        &mut dst,
    );

    dst = ThirdDigit_Rect.clone();
    SDL_UpperBlit(
        EnemyDigitSurfacePointer[usize::try_from(name[2] - b'1' as i8 + 1).unwrap()],
        null_mut(),
        BuildBlock,
        &mut dst,
    );

    // now blit the whole construction to screen:
    if x == -1 {
        let user_center = get_user_center();
        dst.x = (f32::from(user_center.x) + (droid.pos.x - Me.pos.x) * f32::from(Block_Rect.w)
            - f32::from(Block_Rect.w / 2)) as i16;
        dst.y = (f32::from(user_center.y) + (droid.pos.y - Me.pos.y) * f32::from(Block_Rect.h)
            - f32::from(Block_Rect.h / 2)) as i16;
    } else {
        dst.x = x.try_into().unwrap();
        dst.y = y.try_into().unwrap();
    }
    SDL_UpperBlit(BuildBlock, null_mut(), ne_screen, &mut dst);

    //--------------------
    // At this point we can assume, that the enemys has been blittet to the
    // screen, whether it's a friendly enemy or not.
    //
    // So now we can add some text the enemys says.  That might be fun.
    //
    if x == -1
        && droid.TextVisibleTime < GameConfig.WantedTextVisibleTime
        && GameConfig.Droid_Talk != 0
    {
        PutStringFont(
            ne_screen,
            Font0_BFont,
            (f32::from(User_Rect.x)
                + f32::from(User_Rect.w / 2)
                + f32::from(Block_Rect.w / 3)
                + (droid.pos.x - Me.pos.x) * f32::from(Block_Rect.w)) as i32,
            (f32::from(User_Rect.y) + f32::from(User_Rect.h / 2) - f32::from(Block_Rect.h / 2)
                + (droid.pos.y - Me.pos.y) * f32::from(Block_Rect.h)) as i32,
            droid.TextToBeDisplayed,
        );
    }

    info!("ENEMY HAS BEEN PUT --> usual end of function reached.");
}
