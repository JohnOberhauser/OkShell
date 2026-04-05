use zbus::connection;

pub async fn bus_command_with_reply<A: serde::Serialize + zbus::zvariant::Type, R: serde::de::DeserializeOwned + zbus::zvariant::Type>(
    method_name: &'static str,
    arg: &A,
) -> zbus::Result<R> {
    let connection = connection::Builder::session()?.build().await?;
    let reply = connection
        .call_method(
            Some("com.okshell.Shell"),
            "/com/okshell/Shell",
            Some("com.okshell.Shell"),
            method_name,
            arg,
        )
        .await?;
    reply.body().deserialize()
}