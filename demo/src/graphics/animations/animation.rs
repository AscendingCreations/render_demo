use crate::graphics::{allocation::Allocation, AnimationVertex};

//rendering data for all sprites.
//not to be confused with Actual NPC or Player data.
pub struct Animation {
    pub pos: [f32; 3],
    pub hw: [u32; 2],      //render HW
    pub anim_hw: [u32; 2], //image HW per frame
    pub hue_alpha: [u32; 2],
    pub frames: u32,
    pub frames_per_row: u32,
    //in millsecs 1000 = 1sec
    pub switch_time: u32,
    //Texture area location in Atlas.
    pub texture: Option<Allocation>,
    pub bytes: Vec<u8>,
    //if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 1.0],
            hw: [0; 2],
            anim_hw: [0; 2],
            hue_alpha: [0, 100],
            frames: 0,
            frames_per_row: 0,
            switch_time: 0,
            texture: None,
            bytes: Vec::new(),
            changed: true,
        }
    }
}

impl Animation {
    pub fn new(texture: Allocation) -> Self {
        Self {
            pos: [0.0, 0.0, 1.0],
            hw: [0; 2],
            hue_alpha: [0, 100],
            frames: 0,
            frames_per_row: 0,
            switch_time: 0,
            anim_hw: [0; 2],
            texture: Some(texture),
            bytes: Vec::new(),
            changed: true,
        }
    }

    pub fn create_quad(&mut self) {
        let (x, y, w, h) = (
            self.pos[0],
            self.pos[1],
            self.pos[0] + self.hw[0] as f32,
            self.pos[1] + self.hw[1] as f32,
        );

        let allocation = match &self.texture {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v) = allocation.position();

        let (u1, v1, u2, v2) = (0.0, 0.0, self.anim_hw[0] as f32, self.anim_hw[1] as f32);

        let buffer = vec![
            AnimationVertex {
                tex_coord: [u1, v2],
                tex_data: [u, v, self.anim_hw[0], self.anim_hw[1]],
                hue_alpha: self.hue_alpha,
                position: [x, y, self.pos[2] as f32],
                frames: [self.frames, self.frames_per_row, self.switch_time],
                layer: allocation.layer as i32,
            },
            AnimationVertex {
                tex_coord: [u2, v2],
                tex_data: [u, v, self.anim_hw[0], self.anim_hw[1]],
                hue_alpha: self.hue_alpha,
                position: [w, y, self.pos[2] as f32],
                frames: [self.frames, self.frames_per_row, self.switch_time],
                layer: allocation.layer as i32,
            },
            AnimationVertex {
                tex_coord: [u2, v1],
                tex_data: [u, v, self.anim_hw[0], self.anim_hw[1]],
                hue_alpha: self.hue_alpha,
                position: [w, h, self.pos[2] as f32],
                frames: [self.frames, self.frames_per_row, self.switch_time],
                layer: allocation.layer as i32,
            },
            AnimationVertex {
                tex_coord: [u1, v1],
                tex_data: [u, v, self.anim_hw[0], self.anim_hw[1]],
                hue_alpha: self.hue_alpha,
                position: [x, h, self.pos[2] as f32],
                frames: [self.frames, self.frames_per_row, self.switch_time],
                layer: allocation.layer as i32,
            },
        ];

        self.bytes = bytemuck::cast_slice(&buffer).to_vec();
        self.changed = false;
    }

    //used to check and update the vertex array.
    pub fn update(&mut self) {
        //if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad();
        }
    }
}
