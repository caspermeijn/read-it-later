// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use futures::executor::ThreadPool;
use gtk::{
    gio, glib,
    glib::{clone, Sender},
    prelude::*,
    subclass::prelude::*,
};

use crate::models::{Article, ArticleAction, ArticleObject, ArticlesFilter};

mod imp {
    use std::cell::{OnceCell, RefCell};

    use gtk::glib::subclass::InitializingObject;

    use super::*;
    use crate::widgets::articles::ArticlesListWidget;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/articles.ui")]
    pub struct ArticlesView {
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub unread_view: TemplateChild<ArticlesListWidget>,
        #[template_child]
        pub favorites_view: TemplateChild<ArticlesListWidget>,
        #[template_child]
        pub archive_view: TemplateChild<ArticlesListWidget>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,

        pub model: OnceCell<gio::ListStore>,
        pub progress_bar_timeout: RefCell<Option<glib::source::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticlesView {
        const NAME: &'static str = "ArticlesView";
        type Type = super::ArticlesView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticlesView {
        fn constructed(&self) {
            self.parent_constructed();

            let model = gio::ListStore::new::<glib::Object>();

            let filter: gtk::Filter = ArticlesFilter::favorites().into();
            let favorites_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter));
            self.favorites_view.bind_model(&favorites_model);

            let filter: gtk::Filter = ArticlesFilter::archive().into();
            let archive_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter));
            self.archive_view.bind_model(&archive_model);

            let filter: gtk::Filter = ArticlesFilter::unread().into();
            let unread_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter));
            self.unread_view.bind_model(&unread_model);

            self.model.set(model).unwrap();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticlesView {}
}

glib::wrapper! {
    pub struct ArticlesView(ObjectSubclass<imp::ArticlesView>)
        @extends gtk::Widget;
}

impl ArticlesView {
    pub fn set_sender(&self, sender: Sender<ArticleAction>) {
        let imp = self.imp();
        imp.favorites_view.set_sender(sender.clone());
        imp.archive_view.set_sender(sender.clone());
        imp.unread_view.set_sender(sender.clone());

        let articles = Article::load().unwrap();
        let pool = ThreadPool::new().expect("Failed to build pool");

        let ctx = glib::MainContext::default();
        ctx.spawn(async move {
            let futures = async move {
                articles.into_iter().for_each(|article| {
                    sender.send(ArticleAction::Add(article)).unwrap();
                })
            };
            pool.spawn_ok(futures);
        });
    }

    pub fn add(&self, article: &Article) {
        let imp = self.imp();
        let model = imp.model.get().unwrap();
        if self.index(article).is_none() {
            let object = ArticleObject::new(article.clone());
            model.insert_sorted(&object, Article::compare);
        }
    }

    pub fn clear(&self) {
        let imp = self.imp();
        let model = imp.model.get().unwrap();
        model.remove_all();
    }

    pub fn update(&self, article: &Article) {
        self.delete(article);
        self.add(article);
    }

    pub fn delete(&self, article: &Article) {
        let imp = self.imp();
        let model = imp.model.get().unwrap();
        if let Some(pos) = self.index(article) {
            model.remove(pos);
        }
    }

    pub fn favorite(&self, article: &Article) {
        self.update(article);
    }

    pub fn archive(&self, article: &Article) {
        self.update(article);
    }

    fn index(&self, article: &Article) -> Option<u32> {
        let imp = self.imp();
        let model = imp.model.get().unwrap();
        for i in 0..model.n_items() {
            let gobject = model.item(i).unwrap();
            let a = gobject.downcast_ref::<ArticleObject>().unwrap().article();

            if article.id == a.id {
                return Some(i);
            }
        }
        None
    }

    pub fn get_stack(&self) -> &adw::ViewStack {
        &self.imp().stack
    }

    pub fn set_progress_bar_pulsing(&self, state: bool) {
        let imp = self.imp();
        if !state {
            if let Some(timeout) = imp.progress_bar_timeout.replace(None) {
                timeout.remove();
            }
            imp.revealer.set_reveal_child(false);
        } else {
            // Reset the progress bar position
            imp.progress_bar.set_fraction(0.0);
            let timeout = glib::timeout_add_local(
                std::time::Duration::from_millis(100),
                clone!(@weak imp => @default-return glib::ControlFlow::Break, move || {
                    imp.revealer.set_reveal_child(true);
                    imp.progress_bar.pulse();
                    glib::ControlFlow::Continue
                }),
            );
            if let Some(old_timeout) = imp.progress_bar_timeout.replace(Some(timeout)) {
                old_timeout.remove();
            }
        }
    }
}
