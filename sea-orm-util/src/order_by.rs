use sea_orm::{
    sea_query::{Func, SelectStatement, SimpleExpr},
    EntityTrait, Order, QueryOrder, Select,
};

pub trait OrderByRandom {
    fn order_by_random(self) -> Self;
}

impl<E: EntityTrait> OrderByRandom for Select<E> {
    fn order_by_random(mut self) -> Self {
        QueryOrder::query(&mut self).order_by_expr(
            // SimpleExpr::FunctionCall(Function::Custom(Arc::new(Random)), Vec::new()), // 0.9.x
            SimpleExpr::FunctionCall(Func::random()), // 0.11.x
            Order::Desc,
        );

        self
    }
}

impl OrderByRandom for SelectStatement {
    fn order_by_random(mut self) -> Self {
        self.order_by_expr(
            // SimpleExpr::FunctionCall(Function::Custom(Arc::new(Random)), Vec::new()), // 0.9.x
            SimpleExpr::FunctionCall(Func::random()), // 0.11.x
            Order::Desc,
        );

        self
    }
}
