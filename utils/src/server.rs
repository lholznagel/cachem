#[macro_export]
macro_rules! cachem {
    (
        FetchId,
        $model:ty,
        $cache_copy:expr,
        $socket:expr
    ) => {
        {
            let id = Protocol::read::<_, $model>(&mut $socket).await.unwrap();
            if let Some(x) = $cache_copy.fetch_by_id(id.0).await {
                Protocol::response(&mut $socket, x).await
            } else {
                $socket.write_u8(0u8).await.unwrap();
                Ok(())
            }
        }
    };
    (
        FetchAll,
        $model:ty,
        $cache_copy:expr,
        $socket:expr
    ) => {
        {
            if let Some(x) = $cache_copy.fetch_all().await {
                Protocol::response(&mut $socket, x).await
            } else {
                $socket.write_u8(0u8).await.unwrap();
                Ok(())
            }
        }
    };
    (
        Lookup,
        $model:ty,
        $cache_copy:expr,
        $socket:expr
    ) => {
        {
            let data = Protocol::read::<_, $model>(&mut $socket).await.unwrap();
            if let Ok(x) = $cache_copy.lookup(data.0).await {
                Protocol::response(&mut $socket, x).await
            } else {
                Err(CachemError::Empty)
            }
        }
    };
    (
        Insert,
        $model:ty,
        $cache_copy:expr,
        $socket:expr
    ) => {
        {
            let data = Protocol::read::<_, $model>(&mut $socket).await.unwrap();
            if let Ok(_) = $cache_copy.insert(data.0).await {
                Protocol::response(&mut $socket, EmptyResponse::default()).await
            } else {
                Err(CachemError::Empty)
            }
        }
    };
    (
        $uri:expr,
        $(let $v:ident = $e:expr;)*
        $(($action:path, $cache:path) => ($marker:ident, $model:path, $cache_copy:ident),)*
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

                    let action = Actions::from(buf[0]);
                    let cache = Caches::from(buf[1]);
                    let x = match (&action, &cache) {
                        $((&$action, &$cache) => cachem!($marker, $model, $cache_copy, buf_socket),)*
                        _ => panic!("Invalid action / cache combination ({:?}, {:?})", action, cache)
                    };

                    // return the socket so that we don´t consume it
                    socket = buf_socket.into_inner();
                }
            });
        }
    };
}
