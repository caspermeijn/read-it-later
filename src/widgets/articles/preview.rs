use cairo::Context;
use gdk::ContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::*;

use std::cell::RefCell;
use std::f64;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum ArticlePreviewImageType {
    Cover,
    Thumbnail,
}

pub struct ArticlePreviewImage {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    image: gtk::DrawingArea,
    pixbuf: Rc<RefCell<Option<Pixbuf>>>,
    image_type: RefCell<ArticlePreviewImageType>,
}

impl ArticlePreviewImage {
    pub fn new(image_type: ArticlePreviewImageType) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article_preview.ui");
        get_widget!(builder, gtk::Box, article_preview);
        get_widget!(builder, gtk::DrawingArea, image);
        let pixbuf = Rc::new(RefCell::new(None));

        let favicon = Rc::new(Self {
            widget: article_preview,
            builder,
            image,
            pixbuf,
            image_type: RefCell::new(image_type.clone()),
        });
        favicon.setup_signals(favicon.clone());
        favicon.set_image_type(image_type);
        favicon
    }

    pub fn set_image_type(&self, image_type: ArticlePreviewImageType) {
        let ctx = self.widget.get_style_context();
        match image_type {
            ArticlePreviewImageType::Thumbnail => {
                ctx.add_class("preview-thumbnail");
                ctx.remove_class("preview-cover");
            }
            ArticlePreviewImageType::Cover => {
                ctx.add_class("preview-cover");
                ctx.remove_class("preview-thumbnail");
            }
        };
        self.image_type.replace(image_type);
        self.image.queue_draw();
    }

    pub fn set_size(&self, width: i32, height: i32) {
        self.image.set_size_request(width as i32, height as i32);
        self.image.queue_draw();
    }

    pub fn set_pixbuf(&self, pixbuf: Pixbuf) {
        *self.pixbuf.borrow_mut() = Some(pixbuf);
        self.image.queue_draw();
    }

    fn get_mask(&self) -> cairo::ImageSurface {
        let width = self.image.get_allocated_width() as f64;
        let height = self.image.get_allocated_height() as f64;
        let scale_factor = self.image.get_scale_factor() as f64;

        let mask = cairo::ImageSurface::create(cairo::Format::A8, (width * scale_factor) as i32, (height * scale_factor) as i32).unwrap();
        let border_radius = 5.0;
        let cr = Context::new(&mask);
        cr.scale(scale_factor.into(), scale_factor.into());
        self.rounded_rectangle(cr.clone(), 0.0, 0.0, width, height, border_radius);
        cr.fill();

        return mask;
    }

    fn rounded_rectangle(&self, cr: Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
        let arc0: f64 = 0.0;
        let arc1: f64 = f64::consts::PI * 0.5;
        let arc2: f64 = f64::consts::PI;
        let arc3: f64 = f64::consts::PI * 1.5;

        cr.new_sub_path();

        match *self.image_type.borrow() {
            ArticlePreviewImageType::Cover => {
                cr.arc(x + width - radius, y + radius, radius, arc3, arc0);
                cr.arc(x + width, y + height, 0.0, arc0, arc1);

                cr.arc(x, y + height, 0.0, arc1, arc2);
                cr.arc(x + radius, y + radius, radius, arc2, arc3);
            }
            ArticlePreviewImageType::Thumbnail => {
                cr.arc(x + width - radius, y + radius, radius, arc3, arc0);
                cr.arc(x + width - radius, y + height - radius, radius, arc0, arc1);

                cr.arc(x, y + height, 0.0, arc1, arc2);
                cr.arc(x, y, 0.0, arc2, arc3);
            }
        };
        cr.close_path();
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
                    ctx.save();
                    let mask = d.get_mask();
                    match &*d.image_type.borrow() {
                        ArticlePreviewImageType::Cover => {
                            ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                        }
                        ArticlePreviewImageType::Thumbnail => {
                            let x_offset = (width as f64 * scale_factor - pixbuf.get_width() as f64) / 2.0;
                            let y_offset = (height as f64 * scale_factor - pixbuf.get_height() as f64) / 2.0;
                            ctx.set_source_pixbuf(&pixbuf, x_offset, y_offset);
                        }
                    };
                    ctx.scale(1.0 / scale_factor, 1.0 / scale_factor);
                    ctx.mask_surface(&mask, 0.0, 0.0);
                    ctx.restore();
                    gtk::Inhibit(false)
                }
                None => return gtk::Inhibit(false),
            }
        });
    }
}
