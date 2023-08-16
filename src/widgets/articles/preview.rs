use gtk::{gdk_pixbuf::Pixbuf, glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::{cell::RefCell, str::FromStr};

    use glib::once_cell::sync::Lazy;
    use gtk::glib::{clone, subclass::InitializingObject, ParamSpec, Value};
    use url::Url;

    use super::*;
    use crate::models::PreviewImage;

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
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecString::builder("url").build()]);
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

                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(
                        clone!(@weak self as article_preview => async move {
                            match article_preview.get_preview_picture().await {
                                Some(pixbuf) => article_preview.set_pixbuf(&pixbuf),_ => article_preview.obj().set_visible(false),
                            };
                        }
                    ));
                }
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticlePreview {}

    impl ArticlePreview {
        pub async fn get_preview_picture(&self) -> Option<Pixbuf> {
            let url = self.url.borrow().clone();
            match url {
                Some(preview_picture) => {
                    let preview_image = PreviewImage::new(Url::from_str(&preview_picture).ok()?);
                    if !preview_image.exists() {
                        preview_image.download().await.ok()?;
                    }

                    Pixbuf::from_file(&preview_image.cache).ok()
                }
                None => None,
            }
        }

        pub fn set_pixbuf(&self, pixbuf: &Pixbuf) {
            self.image.set_pixbuf(Some(pixbuf));
            self.image.set_visible(true);
            self.spinner.set_visible(false);
        }
    }
}

glib::wrapper! {
    pub struct ArticlePreview(ObjectSubclass<imp::ArticlePreview>)
        @extends gtk::Widget;
}
