// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use glib::{Receiver, Sender};
use gtk::glib;
use log::error;

use super::article::Article;
use crate::application::Action;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ArticleAction {
    Add(Article),
    Delete(Article),
    Archive(Article),
    Favorite(Article),
    Open(Article),
    Update(Article),
}

pub struct ArticlesManager {
    main_sender: Sender<Action>,
    pub sender: Sender<ArticleAction>,
    receiver: RefCell<Option<Receiver<ArticleAction>>>,
}

impl ArticlesManager {
    pub fn new(main_sender: Sender<Action>) -> Rc<Self> {
        let (sender, r) = glib::MainContext::channel(Default::default());
        let receiver = RefCell::new(Some(r));

        let manager = Rc::new(Self {
            main_sender,
            sender,
            receiver,
        });

        manager.init(manager.clone());
        manager
    }

    fn init(&self, manager: Rc<Self>) {
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| manager.do_action(action));
    }

    fn do_action(&self, action: ArticleAction) -> glib::ControlFlow {
        match action {
            ArticleAction::Delete(article) => self.delete(article),
            ArticleAction::Open(article) => self.open(article),
            ArticleAction::Archive(article) => self.archive(article),
            ArticleAction::Favorite(article) => self.favorite(article),
            // Update article values by their ID.
            ArticleAction::Update(article) => self.update(article),
            ArticleAction::Add(article) => self.add(article),
        };
        glib::ControlFlow::Continue
    }

    fn add(&self, article: Article) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::Add(article))))
            .unwrap();
    }

    fn open(&self, article: Article) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::Open(article))))
            .unwrap();
    }

    fn update(&self, article: Article) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::Update(article))))
            .unwrap();
    }

    fn archive(&self, mut article: Article) {
        match article.toggle_archive() {
            Ok(_) => self
                .main_sender
                .send(Action::Articles(Box::new(ArticleAction::Archive(article))))
                .unwrap(),
            Err(err) => error!("Failed to (un)archive the article {}", err),
        }
    }

    fn favorite(&self, mut article: Article) {
        match article.toggle_favorite() {
            Ok(_) => self
                .main_sender
                .send(Action::Articles(Box::new(ArticleAction::Favorite(article))))
                .unwrap(),
            Err(err) => error!("Failed to (un)favorite the article {}", err),
        }
    }

    fn delete(&self, article: Article) {
        match article.delete() {
            Ok(_) => self
                .main_sender
                .send(Action::Articles(Box::new(ArticleAction::Delete(article))))
                .unwrap(),
            Err(err) => error!("Failed to delete the article {}", err),
        }
    }
}
