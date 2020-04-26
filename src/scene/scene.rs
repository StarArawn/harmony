use super::resources;
use legion::prelude::*;
use legion::systems::schedule::Builder;

pub struct Scene {
    pub universe: Universe,
    pub world: World,
    pub game_schedule: Schedule,
}

impl Scene {
    pub fn new(world: Option<World>, schedule_builder: Option<Builder>) -> Self {
        let universe = Universe::new();
        let world = world.unwrap_or(universe.create_world());

        // Add our systems here..
        let game_schedule_builder = schedule_builder.unwrap_or(Schedule::builder());
        let game_schedule = game_schedule_builder.build();

        Scene { world, game_schedule, universe }
    }

    pub(crate) fn update(&mut self, delta_time: f32, resources: &mut Resources) {
        {
            let mut delta = resources.get_mut::<resources::DeltaTime>().unwrap();
            *delta = resources::DeltaTime(delta_time);
        }

        self.game_schedule.execute(&mut self.world, resources);
    }
}
