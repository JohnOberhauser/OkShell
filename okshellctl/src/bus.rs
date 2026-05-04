use zbus::connection;

pub async fn bus_command(method_name: &'static str) -> zbus::Result<()> {
    let connection = connection::Builder::session()?.build().await?;
    connection
        .call_method(
            Some("com.okshell.Shell"),
            "/com/okshell/Shell",
            Some("com.okshell.Shell"),
            method_name,
            &(),
        )
        .await?;
    Ok(())
}

pub async fn bus_command_with_arg<A: serde::Serialize + zbus::zvariant::Type>(
    method_name: &'static str,
    arg: &A,
) -> zbus::Result<()> {
    let connection = connection::Builder::session()?.build().await?;
    connection
        .call_method(
            Some("com.okshell.Shell"),
            "/com/okshell/Shell",
            Some("com.okshell.Shell"),
            method_name,
            arg,
        )
        .await?;
    Ok(())
}

pub async fn bus_command_with_reply<R: serde::de::DeserializeOwned + zbus::zvariant::Type>(
    method_name: &'static str,
) -> zbus::Result<R> {
    let connection = connection::Builder::session()?.build().await?;
    let reply = connection
        .call_method(
            Some("com.okshell.Shell"),
            "/com/okshell/Shell",
            Some("com.okshell.Shell"),
            method_name,
            &(),
        )
        .await?;
    reply.body().deserialize()
}
