use std::sync::Arc;

use sea_orm::{
    sea_query::{Function, SimpleExpr},
    EntityTrait, Order, QueryOrder, Select,
};

use super::function::Random;

pub trait OrderByRandom {
    fn order_by_random(self) -> Self;
}

impl<E: EntityTrait> OrderByRandom for Select<E> {
    fn order_by_random(mut self) -> Self {
        QueryOrder::query(&mut self).order_by_expr(
            SimpleExpr::FunctionCall(Function::Custom(Arc::new(Random)), Vec::new()),
            Order::Desc,
        );

        self
    }
}
