use eframe::egui::{Rect, Response, Ui, Widget};

pub trait PutClipped {
    fn putc(&mut self, rect: Rect, widget: impl Widget) -> Response;
}

impl PutClipped for Ui {
    fn putc(&mut self, rect: Rect, widget: impl Widget) -> Response {
        let clip = rect.intersect(self.clip_rect());

        self.allocate_ui(rect.size(), |child| {
            child.set_clip_rect(clip);
            child.put(rect, widget);
        })
        .response
    }
}
