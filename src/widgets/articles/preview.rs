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
            image,
            pixbuf,
            size: RefCell::new(size.clone()),
        });
        favicon.setup_signals(favicon.clone());
        favicon
    }

    pub fn set_size(&self, size: PreviewImageSize) {
        match size {
            PreviewImageSize::Small => {
                self.image.set_size_request(75, 75);
            }
            PreviewImageSize::Big => {
                self.image.set_size_request(150, 150);
            }
        };
        self.size.replace(size);
        self.image.queue_draw();
    }

    pub fn set_pixbuf(&self, pixbuf: Pixbuf) {
        *self.pixbuf.borrow_mut() = Some(pixbuf);
        self.image.queue_draw();
    }

    fn setup_signals(&self, d: Rc<Self>) {
        self.image.connect_draw(move |dr, ctx| {
            let width = dr.get_allocated_width();
            let height = dr.get_allocated_height();

            let style = dr.get_style_context();
            gtk::render_background(&style, ctx, 0.0, 0.0, width.into(), height.into());
            gtk::render_frame(&style, ctx, 0.0, 0.0, width.into(), height.into());

            match &*d.pixbuf.borrow() {
                Some(pixbuf) => {
                    if pixbuf.get_width() > width {
                        let pixbuf = pixbuf.scale_simple(width, height, gdk_pixbuf::InterpType::Bilinear).unwrap();
                        ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                    } else if pixbuf.get_width() < width {
                        ctx.set_source_pixbuf(&pixbuf, (width - pixbuf.get_width()) as f64, 0.0);
                    } else {
                        ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                    }
                    //ctx.scale(1.0 / scale_factor, 1.0 / scale_factor);
                    ctx.paint();

                    gtk::Inhibit(false)
                }
                None => return gtk::Inhibit(false),
            }
        });
    }
}
