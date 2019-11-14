use glib::{Receiver, Sender};
use std::cell::RefCell;
use std::rc::Rc;

use super::article::Article;
use crate::application::Action;

#[derive(Debug, PartialEq, Clone)]
pub enum ArticleAction {
    Delete(Article),
    Archive(Article),
    Favorite(Article),
    Open(Article),
    Close,
}

pub struct ArticlesManager {
    /*
        Ensures that the articles are synced
        between the local database and the server
    */
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
            ArticleAction::Close => send!(self.main_sender, Action::PreviousView),
        };
        glib::Continue(true)
    }

    fn open(&self, article: Article) {
        send!(self.main_sender, Action::Articles(ArticleAction::Open(article)));
    }

    fn archive(&self, mut article: Article) {
        match article.toggle_archive() {
            Ok(_) => {
                send!(self.main_sender, Action::Articles(ArticleAction::Archive(article)));
            }
            Err(_) => {}
        }
    }

    fn favorite(&self, mut article: Article) {
        match article.toggle_favorite() {
            Ok(_) => {
                send!(self.main_sender, Action::Articles(ArticleAction::Favorite(article)));
            }
            Err(_) => {}
        }
    }

    fn delete(&self, article: Article) {
        match article.delete() {
            Ok(_) => {
                send!(self.main_sender, Action::Articles(ArticleAction::Delete(article)));
            }
            Err(_) => {}
        }
    }
}
