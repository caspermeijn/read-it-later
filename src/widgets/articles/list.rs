use super::row::ArticleRow;
use crate::models::{ArticleAction, ArticleObject};
use gio::prelude::*;
use glib::clone;
use glib::Object;
use glib::Sender;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk_macros::send;
use log::error;
use once_cell::sync::OnceCell;

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;
    use glib::ParamSpec;
    use glib::ParamSpecString;
    use glib::Value;
    use gtk::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/articles_list.ui")]
    pub struct ArticlesListWidget {
        #[template_child]
        pub empty_status: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub articles_listbox: TemplateChild<gtk::ListBox>,
        pub sender: OnceCell<Sender<ArticleAction>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for ArticlesListWidget {
        const NAME: &'static str = "ArticlesListWidget";
        type Type = super::ArticlesListWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticlesListWidget {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![ParamSpecString::builder("placeholder-icon-name").build()]);
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "placeholder-icon-name" => self.empty_status.icon_name().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "placeholder-icon-name" => {
                    let icon_name = value.get().unwrap();
                    self.empty_status.set_icon_name(icon_name)
                }
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ArticlesListWidget {}

    #[gtk::template_callbacks]
    impl ArticlesListWidget {
        #[template_callback]
        fn handle_row_activated(&self, article_row: &ArticleRow, _list_box: &gtk::ListBox) {
            let sender = self.sender.get().unwrap();
            send!(sender, ArticleAction::Open(article_row.article().article().clone()));
        }
    }
}

glib::wrapper! {
    pub struct ArticlesListWidget(ObjectSubclass<imp::ArticlesListWidget>)
        @extends gtk::Widget;
}

impl ArticlesListWidget {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let list_widget: Self = Object::new(&[]);
        list_widget.imp().sender.set(sender).unwrap();
        list_widget
    }

    fn update_model_empty(&self, model: &gio::ListStore) {
        if model.n_items() == 0 {
            self.imp().stack.set_visible_child_name("empty")
        } else {
            self.imp().stack.set_visible_child_name("list")
        }
    }

    pub fn bind_model(&self, model: &gio::ListStore) {
        self.update_model_empty(model);
        model.connect_items_changed(clone!(@strong self as list_widget => move |model, _, _, _| {
            list_widget.update_model_empty(model);
        }));

        self.imp().articles_listbox.bind_model(Some(model), move |article| {
            let article = article.downcast_ref::<ArticleObject>().unwrap();
            let row = ArticleRow::new(article.clone());
            row.upcast::<gtk::Widget>()
        });
    }
}
