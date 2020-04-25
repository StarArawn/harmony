use super::resources;
use legion::prelude::*;
use legion::systems::schedule::Builder;
use crate::graphics;

pub struct Scene {
    pub universe: Universe,
    pub world: World,
    pub game_schedule: Schedule,
    pub render_schedule: Schedule,
    pub resources: Resources,
}

impl Scene {
    pub fn new(world: Option<World>, schedule_builder: Option<Builder>) -> Self {
        let universe = Universe::new();
        let world = world.unwrap_or(universe.create_world());
        
        // Add resources
        let mut resources = Resources::default();
        resources.insert(resources::DeltaTime(0.05));

        // Add our systems here..
        let game_schedule_builder = schedule_builder.unwrap_or(Schedule::builder());
        let render_schedule = Schedule::builder()
            .add_system(graphics::systems::skybox::create())
            .flush()
            .add_thread_local_fn(graphics::systems::render::create())
            .build();

        let game_schedule = game_schedule_builder.build();

        Scene { world, game_schedule, render_schedule, universe, resources }
    }

    pub(crate) fn update(&mut self, delta_time: f32) {
        {
            let mut delta = self.resources.get_mut::<resources::DeltaTime>().unwrap();
            *delta = resources::DeltaTime(delta_time);
        }

        self.game_schedule.execute(&mut self.world, &mut self.resources);
    }

    pub(crate) fn render(&mut self) {
        self.render_schedule.execute(&mut self.world, &mut self.resources);
    }
}
