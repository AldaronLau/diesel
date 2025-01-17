use std::marker::PhantomData;

use crate::backend::Backend;
use crate::query_builder::*;
use crate::result::QueryResult;

pub trait StaticQueryFragment {
    type Component: 'static;
    const STATIC_COMPONENT: &'static Self::Component;
}

#[derive(Debug, Copy, Clone)]
pub struct StaticQueryFragmentInstance<T>(PhantomData<T>);

impl<T> StaticQueryFragmentInstance<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T, DB> QueryFragment<DB> for StaticQueryFragmentInstance<T>
where
    DB: Backend,
    T: StaticQueryFragment,
    T::Component: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, pass: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        T::STATIC_COMPONENT.walk_ast(pass)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Identifier<'a>(pub &'a str);

impl<'a, DB: Backend> QueryFragment<DB> for Identifier<'a> {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.push_identifier(self.0)
    }
}

pub trait MiddleFragment<DB: Backend> {
    fn push_sql(&self, pass: AstPass<'_, '_, DB>);
}

impl<'a, DB: Backend> MiddleFragment<DB> for &'a str {
    fn push_sql(&self, mut pass: AstPass<'_, '_, DB>) {
        pass.push_sql(self);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InfixNode<T, U, M> {
    lhs: T,
    rhs: U,
    middle: M,
}

impl<T, U, M> InfixNode<T, U, M> {
    pub const fn new(lhs: T, rhs: U, middle: M) -> Self {
        InfixNode { lhs, rhs, middle }
    }
}

impl<T, U, DB, M> QueryFragment<DB> for InfixNode<T, U, M>
where
    DB: Backend,
    T: QueryFragment<DB>,
    U: QueryFragment<DB>,
    M: MiddleFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        self.lhs.walk_ast(out.reborrow())?;
        self.middle.push_sql(out.reborrow());
        self.rhs.walk_ast(out.reborrow())?;
        Ok(())
    }
}
