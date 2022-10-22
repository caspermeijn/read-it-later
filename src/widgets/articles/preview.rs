use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk_pixbuf::Pixbuf, glib};

mod imp {
    use super::*;
    use gtk::glib::subclass::InitializingObject;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article_preview.ui")]
    pub struct ArticlePreview {
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,
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
        fn dispose(&self, _obj: &Self::Type) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ArticlePreview {}
}

glib::wrapper! {
    pub struct ArticlePreview(ObjectSubclass<imp::ArticlePreview>)
        @extends gtk::Widget;
}

impl ArticlePreview {
    pub fn set_pixbuf(&self, pixbuf: &Pixbuf) {
        self.imp().image.set_pixbuf(Some(pixbuf));
        self.imp().image.show();
        self.imp().spinner.hide();
    }
}
