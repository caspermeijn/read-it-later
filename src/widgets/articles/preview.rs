use cairo::Context;
use gdk::ContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;

use std::cell::RefCell;
use std::f64;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum PreviewImageSize {
    Small,
    Big,
}

pub struct ArticlePreviewImage {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    image: gtk::DrawingArea,
    pixbuf: Rc<RefCell<Option<Pixbuf>>>,
    size: RefCell<PreviewImageSize>,
}

impl ArticlePreviewImage {
    pub fn new(size: PreviewImageSize) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article_preview.ui");
        get_widget!(builder, gtk::Box, article_preview);
        get_widget!(builder, gtk::DrawingArea, image);
        let pixbuf = Rc::new(RefCell::new(None));

        let favicon = Rc::new(Self {
            widget: article_preview,
            builder,
            image,
            pixbuf,
            size: RefCell::new(size.clone()),
        });
        favicon.setup_signals(favicon.clone());
        favicon
    }

    pub fn set_size(&self, size: PreviewImageSize) {
        let ctx = self.widget.get_style_context();
        let mut width = 0;
        let mut height = 0;
        match size {
            PreviewImageSize::Small => {
                width = 75;
                height = 75;
            }
            PreviewImageSize::Big => {
                width = 150;
                height = 150;
            }
        };
        self.image.set_size_request(width, height);
        self.size.replace(size);
        self.image.queue_draw();
    }

    pub fn set_pixbuf(&self, pixbuf: Pixbuf) {
        *self.pixbuf.borrow_mut() = Some(pixbuf);
        self.image.queue_draw();
    }

    fn setup_signals(&self, d: Rc<Self>) {
        self.image.connect_draw(move |dr, ctx| {
            let scale_factor = dr.get_scale_factor() as f64;

            let width = dr.get_allocated_width();
            let height = dr.get_allocated_height();

            let style = dr.get_style_context();
            gtk::render_background(&style, ctx, 0.0, 0.0, width.into(), height.into());
            gtk::render_frame(&style, ctx, 0.0, 0.0, width.into(), height.into());

            match &*d.pixbuf.borrow() {
                Some(pixbuf) => {
                    let x_offset = (width as f64 * scale_factor - pixbuf.get_width() as f64) / 2.0;
                    let y_offset = (height as f64 * scale_factor - pixbuf.get_height() as f64) / 2.0;

                    //ctx.scale(1.0 / scale_factor, 1.0 / scale_factor);
                    ctx.scale(width as f64 / pixbuf.get_width() as f64, height as f64 / pixbuf.get_height() as f64);
                    //ctx.translate(x_offset, y_offset);

                    ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);

                    ctx.paint();

                    gtk::Inhibit(false)
                }
                None => return gtk::Inhibit(false),
            }
        });
    }
}
