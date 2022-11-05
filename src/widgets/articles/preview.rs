use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk_pixbuf::Pixbuf, glib};

mod imp {
    use super::*;
    use crate::models::PreviewImage;
    use gtk::glib::{clone, subclass::InitializingObject, ParamSpec, Value};
    use gtk_macros::spawn;
    use once_cell::sync::Lazy;
    use std::{cell::RefCell, str::FromStr};
    use url::Url;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article_preview.ui")]
    pub struct ArticlePreview {
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,

        pub url: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticlePreview {
        const NAME: &'static str = "ArticlePreview";
        type Type = super::ArticlePreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticlePreview {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![glib::ParamSpecString::builder("url").build()]);
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "url" => self.url.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "url" => {
                    let url = value.get().unwrap();
                    self.url.replace(url);

                    spawn!(clone!(@weak self as article_preview => async move {
                        match article_preview.get_preview_picture().await {
                            Some(pixbuf) => article_preview.set_pixbuf(&pixbuf),
                            _ => article_preview.hide(),
                        };
                    }));
                }
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ArticlePreview {}

    impl ArticlePreview {
        pub async fn get_preview_picture(&self) -> Option<Pixbuf> {
            if let Some(preview_picture) = self.url.borrow().clone() {
                let preview_image = PreviewImage::new(Url::from_str(&preview_picture).ok()?);
                if !preview_image.exists() {
                    preview_image.download().await.ok()?;
                }

                return Some(Pixbuf::from_file(&preview_image.cache).ok()?);
            }
            None
        }

        pub fn set_pixbuf(&self, pixbuf: &Pixbuf) {
            self.image.set_pixbuf(Some(pixbuf));
            self.image.show();
            self.spinner.hide();
        }
    }
}

glib::wrapper! {
    pub struct ArticlePreview(ObjectSubclass<imp::ArticlePreview>)
        @extends gtk::Widget;
}
