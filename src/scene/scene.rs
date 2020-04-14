use specs::world::WorldExt;
use specs::{ Dispatcher, DispatcherBuilder, World };
use crate::scene::components;

pub struct Scene<'a> {
    pub world: World,
    dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> Scene<'a> {
    pub fn new(world: Option<World>, dispatch_buider: Option<DispatcherBuilder<'a, 'a>>) -> Self {
        // Add our components here
        let mut world = world.unwrap_or(World::new());
        world.register::<components::Mesh>();
        world.register::<components::CameraData>();
        world.register::<components::Transform>();

        // Add our systems here..
        let dispatch_buider = dispatch_buider.unwrap_or(DispatcherBuilder::new());
        
        let dispatcher = dispatch_buider.build();

        Scene {
            world,
            dispatcher,
        }
    }

    pub(crate) fn update(&mut self, _delta_time: f32) {
        self.dispatcher.dispatch(&mut self.world);

        self.world.maintain();
    }
}