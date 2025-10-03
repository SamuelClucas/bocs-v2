use {Stage, StageInjection};

pub struct Backend {
    win: Arc<Window>,
    gfx_ctx: GraphicsContext,
    world: World,
    bridge: Bridge,
    resources: Resources,
    compute: Compute,
    render: Render,
}

impl Stage<Arc<Window>> for Backend {
    fn new(payload: Arc<Window>) ->  Result<Self, Box<dyn Error>> {

        let mut gfx_ctx: GraphicsContext = GraphicsContext::new(payload).await?; // needs async, modify new GraphicsCOntext to return result and await inside new()

        let dims: Dims3 = [200, 200, 200]; // 
        // World contains voxel_grid and camera
        let world = World::new(dims, &gfx_ctx);

        // Bridge holds rand seed and maintains dispatch dims for raymarch and laplacian
        let bridge = Bridge::new(&world.voxel_grid, &gfx_ctx);

        let resources = Resources::new(&dims, &world, &bridge, &mut gfx_ctx);
        
        let compute = Compute::new(&dims, &resources, &gfx_ctx);
        
        let render = Render::new(&resources, &gfx_ctx);
        
        Ok (
            Self { 
                win: win,
                gfx_ctx: gfx_ctx,
                world: world,
                bridge: bridge,
                resources: resources,
                compute: compute,
                render: render,

                init_complete: false,
                read_ping: true,
                dims: dims,
                time: std::time::Instant::now(),

                mouse_pos: None
                }
        )

    }
}