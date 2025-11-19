#[macro_export]
macro_rules! impl_client_methods {
    ($client_name:ident, $entity:ty, $error:ty, $entity_name_snake:ident) => {
        paste::paste! {
            #[allow(dead_code)]
            impl $client_name {
                #[tracing::instrument(skip(self))]
                pub async fn [<get_ $entity_name_snake>](&self, id: String) -> Result<Option<$entity>, $error> {
                    tracing::debug!("Sending request");
                    self.inner.get(id).await.map_err(|e| <$error>::ActorCommunicationError(e.to_string()))
                }

                #[tracing::instrument(skip(self))]
                #[allow(dead_code)]
                pub async fn [<delete_ $entity_name_snake>](&self, id: String) -> Result<(), $error> {
                    tracing::debug!("Sending request");
                    self.inner.delete(id).await.map_err(|e| <$error>::ActorCommunicationError(e.to_string()))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_client_new {
    ($client_name:ident, $entity:ty) => {
        impl $client_name {
            pub fn new(inner: crate::actor_framework::ResourceClient<$entity>) -> Self {
                Self { inner }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_basic_client {
    ($client_name:ident, $entity:ty, $error:ty, $entity_name_snake:ident) => {
        impl_client_new!($client_name, $entity);
        impl_client_methods!($client_name, $entity, $error, $entity_name_snake);
    };
}
