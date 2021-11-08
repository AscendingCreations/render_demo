#[derive(Clone)]
pub struct Allocation {
    pub allocation: guillotiere::Allocation,
    pub layer: usize,
}

impl Allocation {
    pub fn position(&self) -> (u32, u32) {
        let rectangle = &self.allocation.rectangle;

        (rectangle.min.x as u32, rectangle.min.y as u32)
    }

    pub fn rect(&self) -> (u32, u32, u32, u32) {
        let rec = &self.allocation.rectangle;
        let size = rec.size();
        (
            rec.min.x as u32,
            rec.min.y as u32,
            size.width as u32,
            size.height as u32,
        )
    }

    pub fn size(&self) -> (u32, u32) {
        let size = self.allocation.rectangle.size();

        (size.width as u32, size.height as u32)
    }
}
