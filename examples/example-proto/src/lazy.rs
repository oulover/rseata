use std::sync::{Arc, RwLock};
use tonic::service::Interceptor;

#[derive(Clone)]
pub struct GrpcClient<T>(pub T);
impl<T> std::ops::Deref for GrpcClient<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for GrpcClient<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait LazyStateInit: Sized + Clone {
    type Error;
    type Context;
    type InterceptorType: Interceptor + Clone;

    async fn init(
        ctx: &Self::Context,
    ) -> Result<Self, Self::Error>;
}

#[derive(Clone)]
pub struct LazyState<T, I>
where
    T: LazyStateInit<InterceptorType = I>,
    I: Interceptor + Clone,
{
    inner: Arc<Inner<T>>,
}

impl<T, I> LazyState<T, I>
where
    T: LazyStateInit<InterceptorType = I>,
    I: Interceptor + Clone,
{
    pub fn new(context: T::Context) -> Self {
        println!("---------------LazyStateInit");
        LazyState {
            inner: Arc::new(Inner {
                context,
                value: RwLock::new(None),
            }),
        }
    }

    pub async fn get(&self) -> Result<T, T::Error> {
        {
            let rl = self.inner.value.read().unwrap();
            if let Some(state) = Option::clone(&rl) {
                return Ok(state);
            }
        }
        let state = T::init(&self.inner.context).await?;
        let mut wl = self.inner.value.write().unwrap();
        *wl = Some(state.clone());
        Ok(state)
    }
}

struct Inner<T>
where
    T: LazyStateInit,
{
    context: T::Context,
    value: RwLock<Option<T>>,
}