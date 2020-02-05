use glib::{Receiver, Sender};
use std::cell::RefCell;
use std::rc::Rc;

use super::article::Article;
use crate::application::Action;

#[derive(Debug, PartialEq, Clone)]
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
        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
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

    fn do_action(&self, action: ArticleAction) -> glib::Continue {
        match action {
            ArticleAction::Delete(article) => self.delete(article),
            ArticleAction::Open(article) => self.open(article),
            ArticleAction::Archive(article) => self.archive(article),
            ArticleAction::Favorite(article) => self.favorite(article),
            ArticleAction::Update(article) => self.update(article), // Update article values by their ID.
            _ => (),                                                // Do nothing for now
        };
        glib::Continue(true)
    }

    fn open(&self, article: Article) {
        send!(self.main_sender, Action::Articles(Box::new(ArticleAction::Open(article))));
    }

    fn update(&self, article: Article) {
        send!(self.main_sender, Action::Articles(Box::new(ArticleAction::Update(article))));
    }

    fn archive(&self, mut article: Article) {
        match article.toggle_archive() {
            Ok(_) => send!(self.main_sender, Action::Articles(Box::new(ArticleAction::Archive(article)))),
            Err(err) => error!("Failed to (un)archive the article {}", err),
        }
    }

    fn favorite(&self, mut article: Article) {
        match article.toggle_favorite() {
            Ok(_) => send!(self.main_sender, Action::Articles(Box::new(ArticleAction::Favorite(article)))),
            Err(err) => error!("Failed to (un)favorite the article {}", err),
        }
    }

    fn delete(&self, article: Article) {
        match article.delete() {
            Ok(_) => send!(self.main_sender, Action::Articles(Box::new(ArticleAction::Delete(article)))),
            Err(err) => error!("Failed to delete the article {}", err),
        }
    }
}
