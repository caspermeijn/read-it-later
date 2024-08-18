// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{cell::RefCell, rc::Rc};

use async_std::channel::{Receiver, Sender};
use gtk::glib;
use log::error;

use super::article::Article;
use crate::application::Action;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ArticleAction {
    Add(Article),
    AddMultiple(Vec<Article>),
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
        let (sender, r) = async_std::channel::unbounded();
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

        let ctx = glib::MainContext::default();
        ctx.spawn_local(glib::clone!(
            #[strong]
            manager,
            async move {
                while let Ok(action) = receiver.recv().await {
                    manager.do_action(action).await;
                }
            }
        ));
    }

    async fn do_action(&self, action: ArticleAction) {
        match action {
            ArticleAction::Delete(article) => self.delete(article).await,
            ArticleAction::Open(article) => self.open(article).await,
            ArticleAction::Archive(article) => self.archive(article).await,
            ArticleAction::Favorite(article) => self.favorite(article).await,
            // Update article values by their ID.
            ArticleAction::Update(article) => self.update(article).await,
            ArticleAction::Add(article) => self.add(article).await,
            ArticleAction::AddMultiple(articles) => self.add_multiple(articles).await,
        };
    }

    async fn add(&self, article: Article) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::Add(article))))
            .await
            .unwrap();
    }

    async fn add_multiple(&self, articles: Vec<Article>) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::AddMultiple(
                articles,
            ))))
            .await
            .unwrap();
    }

    async fn open(&self, article: Article) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::Open(article))))
            .await
            .unwrap();
    }

    async fn update(&self, article: Article) {
        self.main_sender
            .send(Action::Articles(Box::new(ArticleAction::Update(article))))
            .await
            .unwrap();
    }

    async fn archive(&self, mut article: Article) {
        match article.toggle_archive() {
            Ok(_) => self
                .main_sender
                .send(Action::Articles(Box::new(ArticleAction::Archive(article))))
                .await
                .unwrap(),
            Err(err) => error!("Failed to (un)archive the article {}", err),
        }
    }

    async fn favorite(&self, mut article: Article) {
        match article.toggle_favorite() {
            Ok(_) => self
                .main_sender
                .send(Action::Articles(Box::new(ArticleAction::Favorite(article))))
                .await
                .unwrap(),
            Err(err) => error!("Failed to (un)favorite the article {}", err),
        }
    }

    async fn delete(&self, article: Article) {
        match article.delete() {
            Ok(_) => self
                .main_sender
                .send(Action::Articles(Box::new(ArticleAction::Delete(article))))
                .await
                .unwrap(),
            Err(err) => error!("Failed to delete the article {}", err),
        }
    }
}
