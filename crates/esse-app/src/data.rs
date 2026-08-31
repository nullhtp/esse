//! The three stores, opened once and shared by the screens.
//!
//! Nothing is cached here — the stores read the disk every time (`esse-core`,
//! store docs). This is only about not threading three handles through every
//! constructor.

use std::rc::Rc;

use esse_core::{DataDir, EssayStore, SessionStore, SparkStore};

pub struct Data {
    pub sparks: SparkStore,
    pub essays: EssayStore,
    pub sessions: SessionStore,
}

impl Data {
    pub fn open(dir: &DataDir) -> Rc<Self> {
        Rc::new(Data {
            sparks: SparkStore::new(dir),
            essays: EssayStore::new(dir),
            sessions: SessionStore::new(dir),
        })
    }
}
