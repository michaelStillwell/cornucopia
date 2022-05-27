#![allow(clippy::all)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
pub mod types {}
pub mod queries {
    pub mod module_1 {
        use futures::{StreamExt, TryStreamExt};
        use cornucopia_client::GenericClient;
        #[derive(Debug, Clone, PartialEq)]
        pub struct ExampleQuery {
            pub col1: String,
        }
        pub struct ExampleQueryBorrowed<'a> {
            pub col1: &'a str,
        }
        impl<'a> From<ExampleQueryBorrowed<'a>> for ExampleQuery {
            fn from(ExampleQueryBorrowed { col1 }: ExampleQueryBorrowed<'a>) -> Self {
                Self { col1: col1.into() }
            }
        }
        pub struct ExampleQueryQuery<'a, C: GenericClient, T, const N: usize> {
            client: &'a C,
            params: [&'a (dyn postgres_types::ToSql + Sync); N],
            indexes: &'static [usize; 1],
            query: &'static str,
            mapper: fn(ExampleQueryBorrowed) -> T,
        }
        impl<'a, C, T: 'a, const N: usize> ExampleQueryQuery<'a, C, T, N>
        where
            C: GenericClient,
        {
            pub fn map<R>(
                self,
                mapper: fn(ExampleQueryBorrowed) -> R,
            ) -> ExampleQueryQuery<'a, C, R, N> {
                ExampleQueryQuery {
                    client: self.client,
                    params: self.params,
                    query: self.query,
                    indexes: self.indexes,
                    mapper,
                }
            }
            pub fn extractor<'b>(
                row: &'b tokio_postgres::row::Row,
                indexes: &'static [usize; 1],
            ) -> ExampleQueryBorrowed<'b> {
                ExampleQueryBorrowed {
                    col1: row.get(indexes[0]),
                }
            }
            pub async fn stmt(
                &self,
            ) -> Result<tokio_postgres::Statement, tokio_postgres::Error> {
                self.client.prepare(self.query).await
            }
            pub async fn one(self) -> Result<T, tokio_postgres::Error> {
                let stmt = self.stmt().await?;
                let row = self.client.query_one(&stmt, &self.params).await?;
                Ok((self.mapper)(Self::extractor(&row, self.indexes)))
            }
            pub async fn vec(self) -> Result<Vec<T>, tokio_postgres::Error> {
                self.stream().await?.try_collect().await
            }
            pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
                let stmt = self.stmt().await?;
                Ok(
                    self
                        .client
                        .query_opt(&stmt, &self.params)
                        .await?
                        .map(|row| (self.mapper)(Self::extractor(&row, self.indexes))),
                )
            }
            pub async fn stream(
                self,
            ) -> Result<
                impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'a,
                tokio_postgres::Error,
            > {
                let stmt = self.stmt().await?;
                let stream = self
                    .client
                    .query_raw(&stmt, cornucopia_client::slice_iter(&self.params))
                    .await?
                    .map(move |res| res
                        .map(|row| (self.mapper)(Self::extractor(&row, self.indexes))))
                    .into_stream();
                Ok(stream)
            }
        }
        pub fn example_query<'a, C: GenericClient>(
            client: &'a C,
        ) -> ExampleQueryQuery<'a, C, ExampleQuery, 0> {
            ExampleQueryQuery {
                client,
                params: [],
                query: "SELECT
    *
FROM
    example_table;

",
                indexes: &[0],
                mapper: |it| ExampleQuery::from(it),
            }
        }
    }
}
