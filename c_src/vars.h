/*
 *
 *   Copyright (c) 1994, 2002, 2003  Johannes Prix
 *   Copyright (c) 1994, 2002, 2003  Reinhard Prix
 *
 *
 *  This file is part of Freedroid
 *
 *  Freedroid is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  Freedroid is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Freedroid; see the file COPYING. If not, write to the
 *  Free Software Foundation, Inc., 59 Temple Place, Suite 330, Boston,
 *  MA  02111-1307  USA
 *
 */


/*
 *  _Definitions_ of global variables
 * This file should only be included in main.c, and
 * the variable _declarations_ should be made in global.h under _main_c
 *
 */
extern SDL_Rect OrigBlock_Rect;      // not to be rescaled ever!!
extern SDL_Rect Block_Rect;
extern SDL_Rect Screen_Rect;
extern SDL_Rect User_Rect;
extern SDL_Rect Classic_User_Rect;
extern SDL_Rect Full_User_Rect;
extern SDL_Rect Banner_Rect;
extern SDL_Rect Portrait_Rect;  // for droid-pic display in console
extern SDL_Rect Cons_Droid_Rect;

extern SDL_Rect Menu_Rect;
extern SDL_Rect OptionsMenu_Rect;

extern SDL_Rect OrigDigit_Rect;  	 // not to be rescaled!
extern SDL_Rect Digit_Rect;

extern SDL_Rect Cons_Header_Rect;
extern SDL_Rect Cons_Menu_Rect;
extern SDL_Rect Cons_Text_Rect;
extern SDL_Rect Cons_Menu_Rects[4];

// Startpos + dimensions of Banner-Texts
extern SDL_Rect LeftInfo_Rect;
extern SDL_Rect RightInfo_Rect;

extern SDL_Rect ProgressMeter_Rect;
extern SDL_Rect ProgressBar_Rect;
extern SDL_Rect ProgressText_Rect;

extern int ShipEmptyCounter;	/* counter to Message: you have won(this ship */

extern influence_t Me;

char *InfluenceModeNames[] = {
  "Mobile",
  "Transfer",
  "Weapon",
  "Captured",
  "Complete",
  "Rejected",
  "Logged In",
  "Debriefing",
  "Terminated",
  "Pause",
  "Cheese",
  "Elevator",
  "Briefing",
  "Menu",
  "Victory",
  "Activate",
  "-- OUT --",
  NULL
};


char *Classname[] = {
  "Influence device",
  "Disposal robot",
  "Servant robot",
  "Messenger robot",
  "Maintenance robot",
  "Crew droid",
  "Sentinel droid",
  "Battle droid",
  "Security droid",
  "Command Cyborg",
  NULL
};

char *Classes[] = {
  "influence",
  "disposal",
  "servant",
  "messenger",
  "maintenance",
  "crew",
  "sentinel",
  "battle",
  "security",
  "command",
  "error"
};

char *Shipnames[ALLSHIPS] = {
  "Paradroid",
  "Metahawk",
  "Graftgold",
  NULL
};

char *Alertcolor[AL_LAST] = {
  "green",
  "yellow",
  "amber",
  "red"
};

char *Drivenames[] = {
  "none",
  "tracks",
  "anti-grav",
  "tripedal",
  "wheels",
  "bipedal",
  "error"
};

char *Sensornames[] = {
  " - ",
  "spectral",
  "infra-red",
  "subsonic",
  "ultra-sonic",
  "radar",
  "error"
};

char *Brainnames[] = {
  "none",
  "neutronic",
  "primode",
  "error"
};

char *Weaponnames[] = {      // Bullet-names:
  "none",                    // pulse
  "lasers",                  // single
  "lasers",                  // Military
  "disruptor",               // flash
  "exterminator",            // exterminator
  "laser rifle",             // laser-rifle
  "error"
};


Druidspec Druidmap;

Bulletspec Bulletmap;

blastspec Blastmap[ALLBLASTTYPES];



