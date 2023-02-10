use sea_orm::{
    sea_query::{Func, SimpleExpr},
    EntityTrait, Order, QueryOrder, Select,
};

use super::function::Random;

pub trait OrderByRandom {
    fn order_by_random(self) -> Self;
}

impl<E: EntityTrait> OrderByRandom for Select<E> {
    fn order_by_random(mut self) -> Self {
        QueryOrder::query(&mut self).order_by_expr(
            // SimpleExpr::FunctionCall(Function::Custom(Arc::new(Random)), Vec::new()), // 0.9.x
            SimpleExpr::FunctionCall(Func::cust(Random)), // 0.11.x
            Order::Desc,
        );

        self
    }
}
