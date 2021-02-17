use crate::Parse;

#[macro_export]
macro_rules! cachem {
    (
        $cache_copy:expr,
        $func:ident,
        $model:ty,
        $socket:expr
    ) => {
        {
            let data = Protocol::read::<_, $model>(&mut $socket).await.unwrap();
            match $cache_copy.$func(data).await {
                Ok(x)  => Protocol::response(&mut $socket, x).await,
                Err(x) => Protocol::response(&mut $socket, x).await,
            }
        }
    };
    (
        $uri:expr,
        $(let $v:ident = $e:expr;)*
        $(- $action:path => ($cache_copy:ident, $func:ident, $model:path),)*
    ) => {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind($uri).await?;
        loop {
            let (mut socket, _) = listener.accept().await?;

            $(let $v = $e;)*

            tokio::spawn(async move {
                // Only read the first two bytes, thats all we need to determine
                // what action and what cache should be used
                let mut buf = [0; 2];

                loop {
                    let mut buf_socket = tokio::io::BufStream::new(socket);
                    match buf_socket.read(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    let action = Actions::from(u16::from_be_bytes(buf));
                    let x = match &action {
                        $(&$action => cachem!($cache_copy, $func, $model, buf_socket),)*
                        _ => panic!("Invalid action ({:?})", action)
                    };

                    // return the socket so that we don´t consume it
                    socket = buf_socket.into_inner();
                }
            });
        }
    };
}

/// <T> describe the requesting model, for example a filter
#[async_trait::async_trait]
pub trait Fetch<T: Parse> {
    type Error;
    type Response;
    async fn fetch(&self, input: T) -> Result<Self::Response, Self::Error>;
}

#[async_trait::async_trait]
pub trait Lookup<T: Parse> {
    type Error;
    type Response;
    async fn lookup(&self, input: T) -> Result<Self::Response, Self::Error>;
}

#[async_trait::async_trait]
pub trait Insert<T: Parse> {
    type Error;
    type Response;
    async fn insert(&self, input: T) -> Result<Self::Response, Self::Error>;
}

#[async_trait::async_trait]
pub trait Update<T: Parse> {
    type Error;
    type Response;
    async fn update(&self, input: T) -> Result<Self::Response, Self::Error>;
}

#[async_trait::async_trait]
pub trait Delete<T: Parse> {
    type Error;
    type Response;
    async fn delete(&self, input: T) -> Result<Self::Response, Self::Error>;
}
