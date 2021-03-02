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
            let res = $cache_copy.$func(data).await;
            if let Err(e) = Protocol::response(&mut $socket, res).await {
                log::error!("Error sending message {:?}", e);
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

                    // ping
                    if buf == [255, 255] {
                        // pong
                        buf_socket.write(&[255, 255]).await.unwrap();
                        buf_socket.flush().await.unwrap();
                        socket = buf_socket.into_inner();
                        continue;
                    }

                    let action = Actions::from(u16::from_be_bytes(buf));
                    let x = match &action {
                        $(&$action => cachem!($cache_copy, $func, $model, buf_socket),)*
                        _ => { log::error!("Invalid action ({:?})", action); }
                    };

                    // return the socket so that we don´t consume it
                    socket = buf_socket.into_inner();
                }
            });
        }
    };
}

#[async_trait::async_trait]
pub trait Fetch<T: Parse> {
    type Response;
    async fn fetch(&self, input: T) -> Self::Response;
}

#[async_trait::async_trait]
pub trait Lookup<T: Parse> {
    type Response;
    async fn lookup(&self, input: T) -> Self::Response;
}

#[async_trait::async_trait]
pub trait Insert<T: Parse> {
    type Response;
    async fn insert(&self, input: T) -> Self::Response;
}
