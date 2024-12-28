use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Defines the radius in the center of the screen where automovers cannot spawn
const FREE_ZONE: f32 = 200.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Speed(100.0))
        .insert_resource(Score(0))
        .insert_resource(Lives(3))
        .add_event::<CollisionWithPresentEvent>()
        .add_event::<CollisionWithSnowflakeEvent>()
        .add_systems(Startup, (
            setup_camera,
            initialize_automovers::<Present, 10>,
            initialize_automovers::<Snowflake, 10>,
            initialize_santa,
            initialize_ui,
        ))
        .add_systems(Update, (
            automoving_system,
            bounce_automovers_system,
            move_santa_system,
            detect_collisions_system::<Present, CollisionWithPresentEvent>,
            detect_collisions_system::<Snowflake, CollisionWithSnowflakeEvent>,
            score_points_system.run_if(on_event::<CollisionWithPresentEvent>),
            update_score_ui.run_if(resource_changed::<Score>),
            take_lives_system.run_if(on_event::<CollisionWithSnowflakeEvent>),
            update_lives_ui.run_if(resource_changed::<Lives>),
            speed_up_on_score.run_if(on_event::<CollisionWithPresentEvent>),
        ))
        .add_systems(PostUpdate, (
            remove_entity_on_collission_system::<CollisionWithPresentEvent>,
            remove_entity_on_collission_system::<CollisionWithSnowflakeEvent>,
            win_system,
            loose_system.run_if(resource_changed::<Lives>),
        ))
        .run();
}

fn setup_camera(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = windows.get_single().unwrap();
    commands.spawn((
        Camera2d,
        Transform::from_xyz(primary_window.width() / 2.0, primary_window.height() / 2.0, 0.0),
    ));
}

// Trait to define the sprite path, so we can use it in generic systems
trait HasSpritePath {
    fn sprite_path() -> &'static str;
}

#[derive(Component, Default)]
struct Present;
impl HasSpritePath for Present {
    fn sprite_path() -> &'static str { "present.png" }
}
#[derive(Component, Default)]
struct Snowflake;
impl HasSpritePath for Snowflake {
    fn sprite_path() -> &'static str { "snowflake.png" }
}

#[derive(Component)]
struct AutoMoving(Vec2);
#[derive(Component)]
struct ColliderCircle(f32);


fn initialize_automovers<T: Component + Default + HasSpritePath, const N: usize>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = windows.get_single().unwrap();
    let width = primary_window.width();
    let height = primary_window.height();
    for _ in 0..N {
        // Select a random position that do not fall within the FREE_ZONE in the center
        let (x, y) = loop {
            let x = 32.0 + fastrand::u32(0..width as u32 - 32) as f32;
            let y = 32.0 + fastrand::u32(0..height as u32 - 32) as f32;
            let distance_to_center = ((x - width / 2.0).powf(2.0) + (y - height / 2.0).powf(2.0)).sqrt();
            if distance_to_center > FREE_ZONE {
                break (x, y);
            }
        };
        // Select random direction
        let direction = Vec2::new(fastrand::f32(), fastrand::f32()).normalize();
            
        commands.spawn((
            T::default(),
            Transform::from_xyz(x, y, 0.0),
            Sprite::from_image(asset_server.load(T::sprite_path())),
            AutoMoving(direction),
            ColliderCircle(16.),
        ));
    }
}

// `Speed` is a resource becauese all the automvers, and even the santa, share the same speed.
#[derive(Resource)]
struct Speed(f32);

fn automoving_system(
    time: Res<Time>,
    speed: Res<Speed>,
    mut automovers: Query<(&mut Transform, &AutoMoving)>,
) {
    for (mut transform, automover) in automovers.iter_mut() {
        let direction = automover.0;
        transform.translation.x += direction.x * speed.0 * time.delta_secs();
        transform.translation.y += direction.y * speed.0 * time.delta_secs();
    }
}

// Bounce automovers off the screen
fn bounce_automovers_system(
    mut automovers: Query<(&mut AutoMoving, &Transform)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = windows.get_single().unwrap();
    let width = primary_window.width();
    let height = primary_window.height();
    for (mut automover, transform) in automovers.iter_mut() {
        let x = transform.translation.x;
        let y = transform.translation.y;
        let half_size = 32. / 2.;

        // Bounce off left or right edge
        if x - half_size <= 0.0 || x + half_size >= width {
            automover.0.x = -automover.0.x; // Reverse x velocity
        }

        // Bounce off top or bottom edge
        if y - half_size <= 0.0 || y + half_size >= height {
            automover.0.y = -automover.0.y; // Reverse y velocity
        }
    }
}


#[derive(Component, Default)]
struct Santa;
impl HasSpritePath for Santa {
    fn sprite_path() -> &'static str { "santa.png" }
}

fn initialize_santa(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = windows.get_single().unwrap();
    commands.spawn((
        Santa::default(),
        // Santa spawns in the middle of the screen
        Transform::from_xyz(primary_window.width() / 2.0, primary_window.height() / 2.0, 0.0),
        Sprite::from_image(asset_server.load(Santa::sprite_path())),
        ColliderCircle(16.),
    ));
}

fn move_santa_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    speed: Res<Speed>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut santa: Query<&mut Transform, With<Santa>>,
) {
    let primary_window = windows.get_single().unwrap();
    let width = primary_window.width();
    let height = primary_window.height();

    let mut santa_transform = santa.single_mut();

    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyJ) {
        if santa_transform.translation.x > 32. / 2. {
            santa_transform.translation.x -= speed.0 * time.delta_secs();
        }
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyL) {
        if santa_transform.translation.x < width - 32. / 2. {
            santa_transform.translation.x += speed.0 * time.delta_secs();
        }
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyI) {
        if santa_transform.translation.y < height - 32. / 2. {
            santa_transform.translation.y += speed.0 * time.delta_secs();
        }
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyK) {
        if santa_transform.translation.y > 32. / 2. {
            santa_transform.translation.y -= speed.0 * time.delta_secs();
        }
    }
}

// Trait for generic systems where we only need to know the entity(in this case collision events)
trait WithEntity {
    fn new(entity: Entity) -> Self;
    fn entity(&self) -> Entity;
}

#[derive(Event)]
pub struct CollisionWithPresentEvent(Entity);
impl WithEntity for CollisionWithPresentEvent {
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
    fn entity(&self) -> Entity {
        self.0
    }
}
#[derive(Event)]
pub struct CollisionWithSnowflakeEvent(Entity);
impl WithEntity for CollisionWithSnowflakeEvent {
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
    fn entity(&self) -> Entity {
        self.0
    }
}

fn detect_collisions_system<C: Component, E: Event + WithEntity>(
    mut event_writer: EventWriter<E>,
    objects: Query<(Entity, &Transform, &ColliderCircle), With<C>>,
    santa: Query<(&Transform, &ColliderCircle), With<Santa>>,
) {
    let (santa_transform, santa_collider) = santa.single();
    for (entity, object_transform, object_collider) in objects.iter() {
        let object_position = object_transform.translation;
        let object_radius = object_collider.0;

        if object_position.distance(santa_transform.translation) < (santa_collider.0 + object_radius) * 1.7 {
            event_writer.send(E::new(entity));
        }
    }
}

fn remove_entity_on_collission_system<E: Event + WithEntity>(
    mut commands: Commands,
    mut event_reader: EventReader<E>,
) {
    for event in event_reader.read() {
        commands.entity(event.entity()).despawn();
    }
}

#[derive(Resource)]
struct Score(u32);
#[derive(Resource)]
struct Lives(u32);

#[derive(Component)]
struct UiScoreText;
#[derive(Component)]
struct UiHeart(u32);

fn initialize_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lives: Res<Lives>,
) {
    // Add score label
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
        Text::new("Score: 0"),
        UiScoreText,
    ));
    // Create Hearts
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            column_gap: Val::Px(5.0),
            ..default()
        },
    )).with_children(|parent| {
        for i in 1..=lives.0 {
            parent.spawn((
                Node {
                    width: Val::Px(21.0),
                    height: Val::Px(18.0),
                    ..default()
                },
                ImageNode::new(asset_server.load("heart.png")),
                UiHeart(i),
            ));
        }
    });
}

fn score_points_system(
    mut score: ResMut<Score>,
    mut event_reader: EventReader<CollisionWithPresentEvent>,
) {
    for _ in event_reader.read() {
        score.0 += 1;
    }
}

fn update_score_ui(
    score: Res<Score>,
    mut query: Query<&mut Text, With<UiScoreText>>,
) {
    let mut text = query.single_mut();
    text.0 = format!("Score: {}", score.0);
}

fn take_lives_system(
    mut lives: ResMut<Lives>,
    mut event_reader: EventReader<CollisionWithSnowflakeEvent>,
) {
    for _ in event_reader.read() {
        lives.0 -= 1;
    }
}

fn update_lives_ui(
    mut commands: Commands,
    lives: Res<Lives>,
    mut query: Query<(Entity, &UiHeart), With<UiHeart>>,
) {
    for (entity, ui_heart) in query.iter_mut() {
        if ui_heart.0 > lives.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn speed_up_on_score(
    mut speed: ResMut<Speed>
    , mut event_reader: EventReader<CollisionWithPresentEvent>
) {
    for _ in event_reader.read() {
        speed.0 += 50.0;
    }
}

fn win_system(
    mut exit: EventWriter<AppExit>,
    query: Query<(), With<Present>>,
) {
    if query.is_empty() {
        println!("You win!");
        exit.send(AppExit::Success);
    }
}

fn loose_system(
    mut exit: EventWriter<AppExit>,
    lives: Res<Lives>,
) {
    if lives.0 == 0 {
        println!("You loose!");
        exit.send(AppExit::Success);
    }
}