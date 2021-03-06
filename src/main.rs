

use tcod::colors::*;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};
use std::cmp; 
use rand::Rng;

//TODO: Constants file
// actual size of the window:
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const PLAYER: usize = 0;

//TODO: Dungeon Generator file
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const MAX_ROOM_MONSTERS: i32 = 3;


//TODO: Field of view file
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Permissive0;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

//size of the game map:
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r: 34, g: 32, b: 52, };
const COLOR_LIGHT_WALL: Color = Color { r: 63, g: 63, b: 116, };
const COLOR_DARK_GROUND: Color = Color { r: 34, g: 32, b: 52, };
const COLOR_LIGHT_GROUND: Color = Color { r: 63, g: 63, b: 116, };

const BLACK: Color = Color { r: 0, g: 0, b: 0 };
const GREEN: Color = Color { r: 106, g: 190, b: 48 };
const YELLOW: Color = Color { r: 251, g: 242, b: 54 };
const FORE_COLOR_EXPLORED: Color = Color { r: 63, g: 63, b: 116 };
const FORE_COLOR_VISIBLE: Color = Color { r: 99, g: 155, b: 255};

const CHAR_WALL: char = '#';
const CHAR_FLOOR: char = '.'; 

const LIMIT_FPS: i32 = 20; //20 fps max

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}
//TODO: Tile source file
/// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }


    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        }
    }
}

//TODO: dungeon generator source file
/// A rectangle on the map, used to characterize a room
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

//TODO: object source file
///A generic object
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    name: String,
    blocks: bool,
    alive: bool,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: false,
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

//TODO: dungeon generator source file
fn create_room(room: Rect, map: &mut Map) {
    //go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

//TODO: dungeon generator source file

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    //horizontal tunnel digger
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

//TODO: dungeon generator source file
fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    //vert tunnel digger
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty()
    }
}

//TODO: Map source file
fn make_map(objects: &mut Vec<Object>) -> Map {
    //fill map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            create_room(new_room, &mut map);
            place_objects(new_room, objects);

            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                //this is the first room
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                //otherwise, let's dig a tunnel
                
                //previous room's center:
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            //finally, append the new room to the list
            rooms.push(new_room);
        }
        

    }

    map
}

fn place_objects(room: Rect, objects: &mut Vec<Object>) {
    // choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);

    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        let mut monster = if rand::random::<f32>() < 0.8 {  // 80% chance of getting an orc
            // create an orc
            Object::new(x, y, 'o', "orc", GREEN, true)
        } else {
            Object::new(x, y, 'T', "troll", GREEN, true)
        };

        objects.push(monster);
    }
}

//TODO: Render functions source file:
fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        //recompute FOV if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // go through all the tiles and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let color = match (visible, wall) {
                //outside the field of view
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                //inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let fore_color = match (visible, wall) {
                //outside the field of view
                (false, true) => FORE_COLOR_EXPLORED,
                (false, false) => FORE_COLOR_EXPLORED,
                //inside fov:
                (true, true) => FORE_COLOR_VISIBLE,
                (true, false) => FORE_COLOR_VISIBLE,
            };
            let character = match wall {
                true => CHAR_WALL,
                false => CHAR_FLOOR,
            };
            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                tcod.con 
                    .set_char_background(x, y, color, BackgroundFlag::Set);
                tcod.con
                    .set_char_foreground(x, y, fore_color);
                tcod.con
                    .set_char(x, y, character);
            }
            
            
        }
    }

    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con)
        }
    }

    blit(
        &tcod.con,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}
//TODO: Key handler source file
fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object)-> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true, //exit game

        Key { code: Up, ..} => player.move_by(0, -1, game),
        Key { code: Down, ..} => player.move_by(0, 1, game),
        Key { code: Left, ..} => player.move_by(-1, 0, game),
        Key { code: Right, ..} => player.move_by(1, 0, game),

        _ => {}
    }

    false
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root: Root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("ZonaRL")
        .init();
    
    let mut tcod = Tcod { root, 
                            con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT), 
                            fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
                        };

    let mut player = Object::new(0, 0, '@', "player", YELLOW, true);
    player.alive = true;

    //let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', "npc", YELLOW, true);
 
    let mut objects = vec![player];

    let mut game = Game {
        map: make_map(&mut objects),
    };

    //populate the FOV Map, following the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    //force FOV "recompute" first time thru game loop
    let mut previous_player_position = (-1, -1);

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();

        // render the screen
        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(&mut tcod, &mut game, &objects, fov_recompute);

        tcod.root.flush();

        // handle keys and exit game if neededx`
        let player = &mut objects[PLAYER];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
