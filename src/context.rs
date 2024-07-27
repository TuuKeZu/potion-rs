use std::{any::Any, sync::Arc};

use handlebars::Handlebars;
use minify_html::{minify, Cfg};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::Postgres;
use warp::Filter;

use sqlx::Pool;

use crate::{storage::Storage, utility::merge};


pub type RouterInnerContext = Box<dyn Context + Send + Sync>;


pub trait Context {
    fn box_clone<>(&self) -> Box<dyn Context + Send + Sync>;
    fn as_any<>(&self) -> &dyn Any;
    fn as_any_mut<>(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn Context + Send + Sync>
{
    fn clone(&self) -> Box<dyn Context + Send + Sync> {
        self.box_clone()
    }
}


#[derive(Clone)]
pub struct Router {
    context: Box<dyn Context + Send + Sync>,
    pub storage: Storage
}

impl Router {
    pub fn new(context: Box<dyn Context + Send + Sync>, path: &[&str]) -> Self {
        Self {
            context,
            storage: Storage::from(path)
        }
    }

    pub fn from_existing(context: Box<dyn Context + Send + Sync>, storage: Storage) -> Self {
        Self {
            context,
            storage
        }
    }

    pub fn downcast<U: Context + Send + Sync + 'static>(&self) -> &U {
        self.context.as_any().downcast_ref::<U>().expect("downcasting from Router<dyn Context>")
    }

    pub fn downcast_mut<U: Context + Send + Sync + 'static>(&mut self) -> &mut U {
        self.context.as_any_mut().downcast_mut::<U>().expect("downcasting from Router<dyn Context>")
    }

    pub fn with_context(
        &self,
    ) -> impl Filter<Extract = (Self,), Error = std::convert::Infallible> + Clone {
        let router = Self::from_existing(self.context.box_clone(), self.storage.clone());
        warp::any().map(move || router.clone())
    }
}