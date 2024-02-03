use std::marker::PhantomData;

use sea_orm::{
    sea_query::{Alias, SelectStatement},
    EntityTrait, Select,
};

pub trait FromSubquery {
    fn from_subquery(self, subquery: SelectStatement) -> Self;
}

struct _Select<E>
where
    E: EntityTrait,
{
    query: SelectStatement,
    entity: PhantomData<E>,
}

impl<E> FromSubquery for Select<E>
where
    E: EntityTrait,
{
    fn from_subquery(self, subquery: SelectStatement) -> Self {
        let entity = E::default();
        let mut r = unsafe { std::mem::transmute::<_, _Select<E>>(self) };

        r.query
            .from_clear()
            .from_subquery(subquery, Alias::new(entity.table_name()));

        unsafe { std::mem::transmute::<_, Select<E>>(r) }
    }
}
