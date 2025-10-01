use winit::dpi::PhysicalSize;
use crate::world::voxel_grid::{P2i, P3};

const DY_SENS: f32 = 0.00005;// half as sensitive y given typical display aspect ratio
const DX_SENS: f32 = 0.0001; 

pub struct FPVCamera {
    pub c: P3, // where c is camera pos in world space, 
    pub f: P3, // where f is unit vector from c to world space origin, orthogonal to u and r (u X r)
    pub u: P3, // where u is unit vector up from c, orthogonal to f and r (f x r)
    pub r: P3, // where r is unit vector right from c, orthogonal to f and u (f X u)  
    pub centre: P3, // ASSUMES WINDOW EXISTS ON CAMERA INIT (check state.rs new())
    kf: f32,

    dx_sens: f32,
    dy_sens: f32
}

impl FPVCamera {   
    // BASIC UTILITY FUNCTIONS
    pub fn normalise(a: &P3, mag: &f32) -> P3{
        [a[0] / mag,
        a[1] / mag,
        a[2] / mag]
    }
    pub fn scale(a: &P3, k: &f32) -> P3{
        [a[0] * k, a[1] * k, a[2] * k]
    }
    pub fn negate(a: &P3) -> P3{
        [-a[0], -a[1], -a[2]]
    }
    pub fn add(a: &P3, b: &P3) -> P3{
        [a[0] + b[0],
        a[1] + b[1],
        a[2] + b[2]]
    }
    pub fn decay(a: &P3, b: &f32) -> P3{
        [a[0] - b,
        a[1] - b,
        a[2] - b]
    }
    // returns right-handed, orthogonal vector to a, b
    pub fn cross(a: &P3, b: &P3) -> P3 {
        [
            (a[1] * b[2]) - (a[2] * b[1]), // x i r 
            (a[2] * b[0]) - (a[0] * b[2]), // y j u
            (a[0] * b[1]) - (a[1] * b[0]) // z k f
        ]
    }
    // returns scalar sum of component-wise products of a and b
    pub fn dot(a: &P3, b: &P3) -> f32{
        (a[0]*b[0])+(a[1]*b[1])+(a[2]*b[2])
    }

    pub fn magnitude(input: &P3) -> f32 {
        let square = Self::dot(input, input);
        square.sqrt()
    }
    // this is moving to the compute shader
    pub fn world_to_ruf(&self, input: &P3) -> P3 { // right is x, up is y, forward is z
        let offset = [input[0]-self.c[0], input[1]-self.c[1], input[2]-self.c[2]];
        [
                Self::dot(&offset, &self.r), // right
                Self::dot(&offset, &self.u), // up
                Self::dot(&offset, &self.f) // forward
        ]
    }
    pub fn ruf_to_ru_plane(&self, input: &P3, r_scale: &f32) -> P2i {
        let normalised = FPVCamera::normalise(input, &FPVCamera::magnitude(input));
        let centre_mag = FPVCamera::magnitude(&self.centre); // scale factor for F and U

        let up_multiplier = centre_mag/normalised[2];

        // for 90 eg vertical fov, F and U are 1:1
        // scale u by f coefficient to centre
        let up_pixels = normalised[1] * up_multiplier; 
        let right_pixels =  normalised[0] * up_multiplier * r_scale;

        [ right_pixels as i32, up_pixels as i32]
    }
    pub fn sin(a: P3) -> P3 {
        [a[0].sin(),
        a[1].sin(),
        a[2].sin()]
        
    }
    pub fn cosine(a: P3) -> P3 {
        [a[0].cos(),
        a[1].cos(),
        a[2].cos()]
        
    }



    /// should add velocity and inertia in the Forward direction 
    pub fn handle_w(&mut self) {
        let step_size = 2.0;
        let max_decay = 1.0; // heavier = smaller decay

        let f_coeff = (step_size - (self.weight.exp())).clamp(0.0, 2.0);

        let step = FPVCamera::scale(&self.f, &f_coeff); // scale F
        FPVCamera::add(&self.inertia, &step); // add F direction to inertia

        let decay = (max_decay - (self.weight.exp())).clamp(0.0, 1.0); // decay inertia
        self.inertia = FPVCamera::decay(&self.inertia, &decay);

        FPVCamera::add(&self.c,&self.inertia); // add inertia to camera pos

        // recompute RUF 

    }

    pub fn handle_s(&mut self) {

    }
    pub fn handle_a(&mut self) {

    }
    pub fn handle_d(&mut self) {

    }
    /// Order of rotation matters, so only this function is exposed externally to handle camera rotations
    /// to ensure order is always preserved
    pub fn handle_rotate(&mut self, dx: f32, dy: f32){
        self.rotate_up(dx); 
        self.rotate_right(dy);
        self.orthonormalise();
    }

    /// Rotate about Up vector (i.e. yaw) takes mouse x deltas
    fn rotate_up(&mut self, dx: f32) {
        // Up == Up
        let coef_sin = (dx * self.dx_sens).sin();
        let coef_cos = (dx * self.dx_sens).cos();
        let old_r = self.r.clone();
        let old_f = self.f.clone();
        self.r = FPVCamera::add(&FPVCamera::scale(&old_f, &coef_sin), &FPVCamera::scale(&old_r, &coef_cos));
        self.f = FPVCamera::add(&FPVCamera::scale(&old_r, &-coef_sin), &FPVCamera::scale(&old_f, &coef_cos));
    }

    /// Rotate about Right vector (i.e. pitch) takes mouse y deltas
    fn rotate_right(&mut self, dy: f32) { // CHECK THESE FOR INSITU MESS
        // Right == Right
        let coef_sin = (dy * self.dy_sens).sin();
        let coef_cos = (dy * self.dy_sens).cos();
        let old_u = self.u.clone();
        let old_f = self.f.clone();
        self.u = FPVCamera::add(&FPVCamera::scale(&old_u, &coef_cos), &FPVCamera::scale(&old_f, &-coef_sin));
        self.f = FPVCamera::add(&FPVCamera::scale(&old_f, &coef_cos), &FPVCamera::scale(&old_u, &coef_sin));
    }

    fn orthonormalise(&mut self) {
        self.r = FPVCamera::normalise(&self.r, &FPVCamera::magnitude(&self.r));
        self.u = FPVCamera::normalise(&self.u, &FPVCamera::magnitude(&self.u));

        self.f = FPVCamera::cross(&self.r, &self.u);
        self.f = FPVCamera::normalise(&self.f, &FPVCamera::magnitude(&self.f)); // norm in case r and u not at 90 deg

        self.r = FPVCamera::cross(&self.u, &self.f);
        // normalising just for sureness
        self.r = FPVCamera::normalise(&self.r, &FPVCamera::magnitude(&self.r));

        self.centre = FPVCamera::scale(&self.f, &self.kf);
    }


    pub fn new(p: P3, size: &PhysicalSize<u32>) -> Self {
        

        let mag = Self::magnitude(&p);
        // forward is negative camera p, normalised by its magnitude
        let right = [1.0,0.0,0.0];
        let up = [0.0,1.0,0.0];
        let forward = [0.0,0.0,1.0];

        // f* kf = centre of near plane
        // near-plane edges = (k * kf) +- (max(width, height) * r or u) (r for horizontal, u for vertical)
        // this gives directions for any x, y pixel
        let kf = (size.width.min(size.height)) as f32 / 2.0; // fixes 90 FOV in smaller dimension, given tan(pi/2) = 1
        let centre = FPVCamera::scale(&forward, &kf);

       FPVCamera { 
        c: p,
        f: forward, 
        u: up, 
        r: right,
        centre: centre,
        kf: kf,
        dx_sens: DX_SENS,
        dy_sens: DY_SENS,
        }

    }
    
}
