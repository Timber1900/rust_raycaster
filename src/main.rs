use nannou::prelude::*;
use nannou::winit::event::{DeviceEvent, ElementState, KeyboardInput};

struct Model {
    player: Player,
    moves: Moves,
    boundaries: Vec<Boundary>,
    resolution: i32,
    fov: f32,
    show2D: bool,
}

struct Player {
    pos: Point2,
    look_dir: Vec2,
}

struct Moves {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    clock: bool,
    anti_clock: bool,
}

#[derive(Clone)]
struct Boundary {
    origin: Point2,
    dir: Vec2,
    length: f32,
}

struct Ray {
    origin: Point2,
    dir: Vec2,
    end: Option<Point2>,
    length: Option<f32>,
    luminosity: Option<f32>,
}

impl Moves {
    fn new() -> Moves {
        Moves {
            up: false,
            down: false,
            left: false,
            right: false,
            clock: false,
            anti_clock: false,
        }
    }

    fn update_moves(&mut self, key: KeyboardInput) {
        if let Some(data) = key.virtual_keycode {
            let state = match key.state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };

            match data {
                Key::W => self.up = state,
                Key::A => self.left = state,
                Key::S => self.down = state,
                Key::D => self.right = state,
                Key::Right => self.clock = state,
                Key::Left => self.anti_clock = state,
                _ => {}
            }
        }
    }

    fn update_player(&self, player: &mut Player) {
        let mut update_vec = vec2(0.0, 0.0);
        let mut update_theta = 0.0;

        if self.up {
            update_vec += player.look_dir * 2.5;
        }
        if self.down {
            update_vec -= player.look_dir * 2.5;
        }
        if self.left {
            update_vec -= player.look_dir.perp() * 2.5;
        }
        if self.right {
            update_vec += player.look_dir.perp() * 2.5;
        }
        if self.clock {
            update_theta += 0.05;
        }
        if self.anti_clock {
            update_theta -= 0.05;
        }

        player.update_player_pos(update_vec);
        player.update_player_look_dir(update_theta);
    }
}

impl Player {
    fn show_player(&self, draw: &Draw) {
        draw.ellipse().w_h(10.0, 10.0).xy(self.pos);

        draw.line()
            .start(self.pos)
            .end(self.pos + (50.0 * self.look_dir))
            .weight(2.0)
            .color(RED);
    }

    fn update_player_pos(&mut self, vel: Vec2) {
        self.pos += vel;
    }

    fn update_player_look_dir(&mut self, d_theta: f32) {
        self.look_dir = self.look_dir.rotate(d_theta);
        self.look_dir = self.look_dir.normalize();
    }

    fn new() -> Player {
        Player {
            pos: pt2(0.0, 0.0),
            look_dir: vec2(1.0, 0.0),
        }
    }
}

impl Ray {
    fn new(player: &Player, d_theta: f32) -> Ray {
        Ray {
            origin: player.pos,
            dir: player.look_dir.rotate(d_theta).normalize(),
            end: None,
            length: None,
            luminosity: None,
        }
    }

    fn intersect(&self, boundary: &Boundary, player: &Player) -> Option<(Point2, f32)> {
        let determinant = (self.dir.x * boundary.dir.y) - (boundary.dir.x * self.dir.y);
        let k = (self.dir.x * (self.origin.y - boundary.origin.y))
            - (self.dir.y * (self.origin.x - boundary.origin.x));

        let lambda = (boundary.dir.x * (self.origin.y - boundary.origin.y))
            - (boundary.dir.y * (self.origin.x - boundary.origin.x));

        let k = k / determinant;
        let lambda = lambda / determinant;

        if lambda >= 0.0 && k >= 0.0 && k < boundary.length {
            return Some((
                boundary.origin + k * boundary.dir,
                5000.0 / ((lambda / 5.0) * (lambda / 5.0)) + 0.2,
            ));
        }

        None
    }

    fn show(&self, draw: &Draw) {
        match self.end {
            Some(point) => {
                draw.line()
                    .start(self.origin)
                    .end(point)
                    .weight(1.0)
                    .color(BLUE);
            }
            None => {
                draw.line()
                    .start(self.origin)
                    .end(self.origin + 1000.0 * self.dir)
                    .weight(1.0)
                    .color(BLUE);
            }
        }
    }
}

impl Boundary {
    fn new(start: Point2, end: Point2) -> Boundary {
        Boundary {
            origin: start,
            dir: (end - start).normalize(),
            length: (end - start).length(),
        }
    }

    fn from_rect(rect: Rect) -> Vec<Boundary> {
        let mut return_val: Vec<Boundary> = Vec::new();

        return_val.push(Boundary::new(
            pt2(rect.x.start, rect.y.start),
            pt2(rect.x.start, rect.y.end),
        ));
        return_val.push(Boundary::new(
            pt2(rect.x.start, rect.y.start),
            pt2(rect.x.end, rect.y.start),
        ));
        return_val.push(Boundary::new(
            pt2(rect.x.end, rect.y.end),
            pt2(rect.x.end, rect.y.start),
        ));
        return_val.push(Boundary::new(
            pt2(rect.x.end, rect.y.end),
            pt2(rect.x.start, rect.y.end),
        ));

        return_val
    }

    fn show(&self, draw: &Draw) {
        draw.line()
            .start(self.origin)
            .end(self.origin + self.length * self.dir)
            .weight(4.0)
            .color(BLACK);
    }
}

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .simple_window(view)
        .run();
}

fn model(app: &App) -> Model {
    let mut boundaries: Vec<Boundary> = Vec::new();

    let new_bounds = Boundary::from_rect(app.window_rect());

    boundaries.extend_from_slice(&new_bounds);

    Model {
        player: Player::new(),
        moves: Moves::new(),
        boundaries,
        resolution: 5,
        fov: 60.0,
        show2D: false,
    }
}

fn event(_app: &App, model: &mut Model, event: Event) {
    if let Event::DeviceEvent(_, data) = event {
        if let DeviceEvent::Key(key) = data {
            model.moves.update_moves(key);
        }
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.moves.update_player(&mut model.player);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let boundaries = app.window_rect();

    draw.background().color(PLUM);

    for i in
        (boundaries.x.start as i32 / model.resolution)..(boundaries.x.end as i32 / model.resolution)
    {
        let angle = (i as f32) / (boundaries.x.end / (model.resolution as f32));
        let angle = map_range(
            angle,
            -1.0,
            1.0,
            -((model.fov * 3.14159265) / (2.0 * 180.0)),
            (model.fov * 3.14159265) / (2.0 * 180.0),
        );

        let mut ray = Ray::new(&model.player, angle);

        for boundary in &model.boundaries {
            let new_point = ray.intersect(&boundary, &model.player);

            if let Some((point, luminosity)) = new_point {
                match ray.end {
                    Some(end) => {
                        if (point - ray.origin).length() < (end - ray.origin).length() {
                            ray.end = Some(point);
                            ray.length = Some((point - ray.origin).length());
                            ray.luminosity = Some(luminosity);
                        }
                    }
                    None => {
                        ray.end = Some(point);
                        ray.length = Some((point - ray.origin).length());
                        ray.luminosity = Some(luminosity);
                    }
                }
            }
        }

        if model.show2D {
            ray.show(&draw);
        } else {
            let x = i * model.resolution;

            let height = match ray.length {
                Some(length) => 100000.0 / (length * angle.cos()),
                None => 0.0,
            };

            let light = match ray.luminosity {
                Some(lum) => {
                    if lum > 0.9 {
                        0.9
                    } else {
                        lum
                    }
                }
                None => 0.0,
            };

            draw.rect()
                .x(x as f32)
                .w_h(model.resolution as f32, height)
                .color(rgba(
                    light,
                    light,
                    light,
                    map_range(light, 0.9, 0.2, 1.0, 0.0),
                ));
        }
    }

    if model.show2D {
        for boundary in &model.boundaries {
            boundary.show(&draw);
        }

        model.player.show_player(&draw);
    }

    draw.to_frame(app, &frame).unwrap();
}
