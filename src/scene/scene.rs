use specs::world::WorldExt;
use specs::{ Dispatcher, DispatcherBuilder, World };
use super::components;
use super::resources;

pub struct Scene<'a> {
    pub world: World,
    dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> Scene<'a> {
    pub fn new(world: Option<World>, dispatch_buider: Option<DispatcherBuilder<'a, 'a>>) -> Self {
        // Add our components here
        let mut world = world.unwrap_or(World::new());
        world.insert(resources::DeltaTime(0.05));
        world.register::<components::Mesh>();
        world.register::<components::Material>();
        world.register::<components::CameraData>();
        world.register::<components::Transform>();
        world.register::<components::SkyboxData>();

        // Add our systems here..
        let dispatch_buider = dispatch_buider.unwrap_or(DispatcherBuilder::new());
        
        let dispatcher = dispatch_buider.build();

        Scene {
            world,
            dispatcher,
        }
    }

    pub(crate) fn update(&mut self, delta_time: f32) {
        {
            let mut delta = self.world.write_resource::<resources::DeltaTime>();
            *delta = resources::DeltaTime(delta_time);
        }

        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
    }
}