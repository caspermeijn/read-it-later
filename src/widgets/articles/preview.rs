use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;

pub struct ArticlePreviewImage {
    pub widget: gtk::Stack,
    image: gtk::DrawingArea,
    pixbuf: Rc<RefCell<Option<Pixbuf>>>,
}

impl ArticlePreviewImage {
    pub fn new() -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/article_preview.ui");
        get_widget!(builder, gtk::Stack, article_preview);
        get_widget!(builder, gtk::DrawingArea, image);
        let pixbuf = Rc::new(RefCell::new(None));

        let favicon = Rc::new(Self {
            widget: article_preview,
            image,
            pixbuf,
        });
        favicon.setup_signals(favicon.clone());
        favicon
    }

    pub fn set_pixbuf(&self, pixbuf: Pixbuf) {
        *self.pixbuf.borrow_mut() = Some(pixbuf);
        self.image.queue_draw();
        self.widget.set_visible_child_name("image");
    }

    fn setup_signals(&self, d: Rc<Self>) {
        self.widget.set_visible_child_name("loading");

        self.image.connect_draw(move |dr, ctx| {
            let width = dr.allocated_width();
            let height = dr.allocated_height();

            let style = dr.style_context();
            gtk::render_background(&style, ctx, 0.0, 0.0, width.into(), height.into());
            gtk::render_frame(&style, ctx, 0.0, 0.0, width.into(), height.into());

            match &*d.pixbuf.borrow() {
                Some(pixbuf) => {
                    match pixbuf.width().cmp(&width) {
                        std::cmp::Ordering::Greater => {
                            let pixbuf = pixbuf.scale_simple(width, height, gdk_pixbuf::InterpType::Bilinear).unwrap();
                            ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                        }
                        std::cmp::Ordering::Less => {
                            ctx.set_source_pixbuf(&pixbuf, (width - pixbuf.width()) as f64, 0.0);
                        }
                        std::cmp::Ordering::Equal => {
                            ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                        }
                    };
                    ctx.paint().unwrap();

                    gtk::Inhibit(false)
                }
                None => gtk::Inhibit(false),
            }
        });
    }
}
