use super::resources;
use legion::prelude::*;
use legion::systems::schedule::Builder;

/// A representation of our scene.
pub struct Scene {
    /// TODO: Move universe out of here and into app?
    pub universe: Universe,
    /// A legion world which contains our entities.
    pub world: World,
    /// A legion schedule for any game related systems.
    pub game_schedule: Schedule,
}

impl Scene {
    /// Allows you to create a new scene with an optional world and optional schedule(which contains systems).
    /// If None is passed in for world or schedule_builder a default one is created.
    pub fn new(world: Option<World>, schedule_builder: Option<Builder>) -> Self {
        let universe = Universe::new();
        let world = world.unwrap_or(universe.create_world());

        // Add our systems here..
        let game_schedule_builder = schedule_builder.unwrap_or(Schedule::builder())
            .add_system(super::systems::culling::create());
        let game_schedule = game_schedule_builder.build();

        Scene {
            world,
            game_schedule,
            universe,
        }
    }

    pub(crate) fn update(&mut self, delta_time: f32, resources: &mut Resources) {
        {
            let mut delta = resources.get_mut::<resources::DeltaTime>().unwrap();
            *delta = resources::DeltaTime(delta_time);
        }

        self.game_schedule.execute(&mut self.world, resources);
    }
}
